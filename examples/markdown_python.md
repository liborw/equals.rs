# equals.rs Markdown (Python) demo

This document mixes prose and evaluated code blocks.

Start with a simple inline snippet: `2 + 3 #= 5` should render the result.

```python
x = 20
y = 4
x / y #= 5.0
```

We can also keep comments that stay at the end of the line:

```python
items = [1, 2, 3, 4]
len(items) #= 4 # bogus value that should be replaced
sum(items) #= 10
```
