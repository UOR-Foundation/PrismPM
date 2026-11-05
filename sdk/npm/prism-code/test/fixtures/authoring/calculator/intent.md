# What the calculator must always and never do

add always returns the exact sum of left and right when the sum fits in 64 signed bits.
subtract always returns the exact difference of left and right when the difference fits in 64 signed bits.
multiply always returns the exact product of left and right when the product fits in 64 signed bits.
divide always truncates the exact quotient toward zero when right is not zero and the quotient fits in 64 signed bits.
calculate never returns a number when the exact result does not fit in 64 signed bits.
calculate returns `division-by-zero` only when the input contains `divide`.
calculate returns `unknown-operation` only when the input contains `\t`.
A number with a leading zero never returns `ok` when the input contains `\t01\t`.
Dividing one by zero never returns `ok` when the input contains `\tdivide\t1\t0`.
