# :parrot: kaw

`kaw` allows you to process input line-by-line using JavaScript expressions. It is inspired by awk but uses the JavaScript methods you're already familiar with. It is written in Rust and powered by [`deno_core`](https://github.com/denoland/deno_core), the foundation for `deno`.

## Usage

`kaw` accepts a single argument, a JavaScript expression to manipulate lines of stdin. A `stdin` variable is available for use in the expression. It is an [`Iterable`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Iterators_and_generators#iterables) that yields the stdin passed to `kaw`, one line at a time (not including newline terminators). `kaw` executes the expression with V8, which has implemented [iterator helpers](https://v8.dev/features/iterator-helpers). This means that you can use methods on `stdin` like `.filter` or `.map` that you have probably used before with arrays. If the expression evaluates to an array or an iterator, it prints one line to stdout for each item in the array or iterator. If the expression evaluates to any other type, it simply converts it to a string and prints the result on a single line.

## Examples

```sh
# Print to stdout every line that is longer than 10 characters
cat input.txt | kaw 'stdin.filter(line => line.length > 10)'

# Print to stdout every line that starts with a #
cat input.txt | kaw 'stdin.filter(line => /^#/.test(line))'

# Print the line number with along with each line
cat input.txt | kaw 'stdin.map((line, index) => `${index + 1} line`)'

# Print 5 lines after the first 10 lines
cat input.txt | kaw 'stdin.drop(10).take(5)'

# Split each line into two lines
cat input.txt | kaw 'stdin.flatMap(line => [line.slice(0, line.length / 2), line.slice(line.length / 2)])'

# Count the number of characters on all lines
cat input.txt | kaw 'stdin.reduce((total, line) => total + line.length, 0)'

# Reverse the lines
cat input.txt | kaw 'stdin.toArray().toReversed()'

# Run multiple transformations on the input lines
cat input.txt | kaw 'stdin.filter(line => line.length > 10).map((line, index) => `${index + 1} line`).take(10)'
```

With the power of V8, you can use any JavaScript feature in your `kaw` expressions, including JSON deserialization, Math functions, dates, or even BigInts.
