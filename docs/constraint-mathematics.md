# Constraint mathematics

Vocabulary and formulation for the shared illustrator constraint runtime.
Behavioral contracts live in OpenSpec capabilities (`geometry-ir`, `state-vector`,
`system-assembly`, `constraint-catalog`, `solve-interaction`, `solver-port`,
`system-architecture`, `display-mesh`). This document aligns the math terms with those specs.

## Glossary

| Term | Meaning |
|------|---------|
| **DocumentStructure** | Discrete sketch: geometries, structural constraints, and the connectivity schema that maps local parameters into indices of a state vector. Independent of GUI and of any concrete solver algorithm. |
| **State** $p$ | Contiguous vector of continuous scalars (degrees of freedom). One State length matches one DocumentStructure's parameter count. |
| **Document** | Composite of DocumentStructure plus a **committed** State. |
| **Vec2 / free vector** | Displacement or direction in the plane. Fundamental linear quantity: difference of points, segment direction, etc. |
| **Point2 / point** | Affine location. May be written as a position vector relative to an origin, but is not a free vector. $P - Q$ is a vector; $P + v$ is a point; $P + Q$ is not meaningful. |
| **World coordinates** | Durable planar frame for Document State and Mesh edge positions. **y-up** (scientific/mathematical): $+y$ points upward. Not window pixels. |
| **Viewport** | App/UI session mapping between world and screen coordinates: uniform scale, origin, and y-flip for toolkits with $+y$ downward. Paint uses world→screen; pointer input and pixel pick radii use screen→world (`r_{\mathrm{world}}=r_{\mathrm{px}}/\mathrm{scale}`). |
| **Point expression** | Map from geometry parameters to a 2D **point** (e.g. a segment endpoint or an on-curve sample). Not a free-standing point entity and not a snapshot detached from parameters. |
| **Vector expression** | Map from geometry parameters to a free **vector** (e.g. segment direction, curve tangent or normal). |
| **Normalized curve parameter** | Chart coordinate \(t \in [0,1]\): the curve's native formula domain affinely mapped to the unit interval. Equal \(\Delta t\) need not mean equal arc length. |
| **Uniform curve parameter** | Arc-length fraction \(u \in [0,1]\): equal \(\Delta u\) means equal arc length along the curve. For a line segment, Normalized and Uniform coincide. |
| **Curve tangent** | Chart derivative \(\gamma'(t)\) as a free vector; length is parametric speed (segment: segment length). Not required to be unit length. |
| **Curve normal** | Leftward planar rotate of the tangent \((-v_y, v_x)\); same length as the tangent. Not a curvature-weighted vector. |
| **Residual** | Scalar (or small vector) that is zero iff a constraint (or soft term) is satisfied within tolerance. |
| **Local Jacobian** | Derivatives of a local residual contribution w.r.t. the local parameters that contribution depends on. |
| **Connectivity** $M_N$ | Explicit map from local parameter indices of contribution $N$ to global indices in $p$. Used to scatter local residuals/Jacobians into global arrays. |
| **Assembly** | Building the global residual $r(p)$ and Jacobian $J(p)=\partial r/\partial p$ by evaluating each contribution and scattering via connectivity. |
| **Soft contribution** | Transient residual used only during Interact (e.g. mouse follow, stay-near-previous). Not stored as a structural constraint in DocumentStructure. |
| **Mesh** | View-layer tessellation of DocumentStructure evaluated at a State \(p\) into drawable edges (tagged with geometry identity and role) in **world coordinates**. Derived only — not authoritative for parameters or constraints. Used for Canvas paint (via Viewport) and optional WYSIWYG hit-testing. Built with a **MeshFilter** that may omit construction geometries. |
| **GeometryRole** | DocumentStructure flag on each geometry: **Result** (published figure) or **Construction** (helper). Construction remains in State and constraints; role alone does not remove parameters. |
| **Presentation mode** | App/UI session choice: result-only vs include-construction. Selects the MeshFilter for paint, Mesh hits, and logical handles so invisible construction is not pickable. |
| **Working copy** | Temporary State used during Interact / solve iterations. May differ from the committed State. |
| **Commit** | Replacing the Document's committed State with a working State after a successful solve (structural residuals within tolerance). |

## Structure versus values

DocumentStructure answers *what* exists and *how* it is wired.
State answers *where* continuous parameters currently sit.

$$
\text{Document} = (\text{DocumentStructure},\; p_{\mathrm{committed}})
$$

Evaluation of geometry and constraints always takes an explicit State $p$
(committed or working), so Interact can try candidates without mutating structure.

## Local geometry maps

A geometry $G$ has local parameters $p_{\mathrm{loc}} \in \mathbb{R}^{n_G}$
identified with a block of the global State via connectivity.

Example (line segment): local parameters
$(x_a,y_a,x_b,y_b)$. Endpoint expressions:

$$
\begin{aligned}
E_{\mathrm{start}}(p_{\mathrm{loc}}) &= (x_a, y_a), \\
E_{\mathrm{end}}(p_{\mathrm{loc}}) &= (x_b, y_b).
\end{aligned}
$$

Analytical derivatives of these maps w.r.t. $p_{\mathrm{loc}}$ feed constraint
Jacobians (see `geometry-eval`).

## Constraints as residual factories

A structural constraint $C$ produces a local residual $r_N$ of fixed arity.
Examples from the v1 catalog:

- **Congruence** of two point expressions $A(p), B(p)$:  
  $r = A(p) - B(p) \in \mathbb{R}^2$.
- **Parallel** of two segment direction vectors $u,v$: one scalar measuring linear dependence (e.g. the 2D cross $u_x v_y - u_y v_x$).
- **Orthogonal**: one scalar $u \cdot v$ (dot of free vectors).
- **FixPoint** of expression $A(p)$ to target $t$:  
  $r = A(p) - t \in \mathbb{R}^2$.

Each contribution also exposes a local Jacobian $J_N = \partial r_N / \partial p_{\mathrm{loc}}$.

## Assembly (FEM-style)

Let contributions $N = 1,\ldots,M$ have local residuals $r_N$ and connectivities $M_N$.
The global residual stacks (scatters) them:

$$
r(p) = \operatorname{assemble}\bigl(r_1(p),\ldots,r_M(p);\; M_1,\ldots,M_M\bigr) \in \mathbb{R}^{m}.
$$

The global Jacobian is the scatter of local blocks:

$$
J[M_N^{\mathrm{row}}(i),\, M_N^{\mathrm{col}}(j)] \mathrel{+}= (J_N)_{ij}.
$$

Because the system is nonlinear, $r$ and $J$ are re-evaluated at the current $p$
whenever the solver requests them (`system-assembly`).

## Soft terms and Interact

During a drag (or similar manipulation), Interact forms an augmented residual

$$
r_{\mathrm{interact}}(p) = \begin{bmatrix} r_{\mathrm{structural}}(p) \\ r_{\mathrm{soft}}(p) \end{bmatrix},
$$

where $r_{\mathrm{soft}}$ may include:

- mouse-follow toward a pointer (weighted),
- stay-near-previous toward the last consistent State.

Structural parts are **hard** for commit: a successful commit requires
$\|r_{\mathrm{structural}}(p)\|$ within solve tolerance. Soft parts are
best-effort and are discarded when the gesture ends.

## What is minimized

The SolverPort drives updates to a working State by repeatedly evaluating
$r(p)$ and $J(p)$ (or an agreed equivalent). Typical owned backends minimize
a nonlinear least-squares objective such as

$$
\min_p \; \tfrac{1}{2}\,\|r_{\mathrm{interact}}(p)\|_2^2
$$

(e.g. Levenberg–Marquardt), but specs do **not** mandate a named algorithm—
only the interaction contract (invariants, soft drag, fail-keeps-committed-state).

On success, Interact **commits** the working State into the Document.
On failure, the committed State is left unchanged.

## Structural edits and $p$

Adding a geometry appends its parameter block to State and records connectivity.
Removing a geometry rebuilds/compacts State, remaps remaining connectivities, and
drops constraints that referenced the removed geometry id. Constraints store
stable geometry identities; raw indices in $p$ are always derived via connectivity.

## See also

- OpenSpec: `system-architecture`, `state-vector`, `geometry-ir`, `geometry-eval`,
  `system-assembly`, `constraint-catalog`, `solve-interaction`, `solver-port`
- Implementation (reference): `impls/linux-rust/crates/constraint_runtime`
