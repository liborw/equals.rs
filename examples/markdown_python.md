# equals.rs Markdown (Python) demo

This document mixes prose and evaluated code blocks.

Start with a simple inline snippet: `2 + 3 #=` should render the result.

```python
x = 10
y = 4
x / y #=
```

We can also keep comments that stay at the end of the line:

```python
items = [1, 2, 3, 4]
len(items) #= 999  # bogus value that should be replaced
sum(items) #=
```
