% Reference values for the expression language (crate `expression`).
%
% Run with GNU Octave from this directory:  octave-cli generate.m
% Writes reference.tsv: source <TAB> x <TAB> value, where value is the real
% result printed with 17 significant digits, or `domain-error` when the
% result is complex, not finite, or an error. Octave is used for values only;
% which expressions are accepted is decided by the expression-language spec.

cases = {
  % precedence and tokens
  '2^3^2', 0;
  '-x^2', 3;
  '2^-1', 0;
  '2*-3', 0;
  % '2--3' (= 5 in MATLAB) is omitted: Octave reads `--` as its decrement
  % operator and fails. The parser tests pin the MATLAB value instead.
  '-2^-2', 0;
  '1-2-3', 0;
  '12/3/2', 0;
  '2+3*4^2', 0;
  '(2^-2)^2', 0;
  '2^(-2)^2', 0;
  '+x', 4;
  'x.^2./4', 3;
  '5.*x', 2;
  '.5 + 5. + 1e-3 + 1.5E+3', 0;
  'x .* sin(x) + 2^-1', 1;
  % catalog functions at interior points
  'exp(x)', 0.7;
  'log(x)', 2.5;
  'log10(x)', 2.5;
  'log2(x)', 2.5;
  'sqrt(x)', 2;
  'sin(x)', 1.2;
  'cos(x)', 1.2;
  'tan(x)', 1.2;
  'asin(x)', 0.4;
  'acos(x)', 0.4;
  'atan(x)', 3;
  'sinh(x)', 1.5;
  'cosh(x)', 1.5;
  'tanh(x)', 0.5;
  'abs(x)', -2.5;
  'nthroot(x, 3)', -8;
  'nthroot(x, 3)', 10;
  'nthroot(x, 4)', 81;
  'nthroot(x, -3)', 8;
  'nthroot(x, 5)', -3.7;
  'atan2(x, -1)', 0.5;
  'atan2(1, x)', -2;
  'hypot(x, 2)', 1.5;
  'pi*x', 2;
  % powers
  'x^x', 1.7;
  '2^x', -1.3;
  'x^2.5', 3;
  'x^-2', -1.5;
  'x^3', -2;
  'x^(1/3)', 8;
  % compositions
  'x*sin(x)', 2;
  'exp(-x^2)/(1+x^2)', 0.8;
  'log(1+x)./x', 0.3;
  'sqrt(x.^2+1)', -3;
  'sin(x)^2+cos(x)^2', 0.9;
  'atan(x)/x', 10;
  'tanh(x)^3', -0.4;
  'cosh(x) - sinh(x)', 3;
  '1e-3*x', 5;
  'x/3', 1;
  % domain errors
  'sqrt(x)', -1;
  'log(x)', -1;
  'log(x)', 0;
  'log10(x)', 0;
  '(-8)^(1/3)', 0;
  'x^(1/3)', -8;
  'asin(x)', 2;
  'acos(x)', -2;
  '1/x', 0;
  'x^-1', 0;
  'nthroot(x, 2)', -4;
  'exp(x)', 1000;
};

dir = fileparts(mfilename('fullpath'));
fid = fopen(fullfile(dir, 'reference.tsv'), 'w');
fprintf(fid, 'source\tx\tvalue\n');
for k = 1:rows(cases)
  src = cases{k, 1};
  x = cases{k, 2};
  try
    f = str2func(['@(x) ' src]);
    v = f(x);
    if ~isreal(v) || ~isfinite(v)
      out = 'domain-error';
    else
      out = sprintf('%.17g', v);
    end
  catch
    out = 'domain-error';
  end
  fprintf(fid, '%s\t%.17g\t%s\n', src, x, out);
end
fclose(fid);
printf("%d rows written\n", rows(cases));
