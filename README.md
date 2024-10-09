# :parrot: kaw

`kaw` is a fast, lightweight tool for processing input line-by-line using JavaScript expressions. It is designed to be a general purpose alternative for commands that manipulate stdin including `awk`, `sed`, `grep`, and `head`. Instead of writing complicated `awk` expressions or digging through man pages, you can use the intuitive JavaScript syntax and methods like `.filter` and `.map` that you're already familiar with.

## Installation

Install `kaw` via Homebrew.

```sh
brew install canac/tap/kaw
```

## Basic usage

```sh
# Print every line that is longer than 10 characters
cat input.txt | kaw 'stdin.filter(line => line.length > 10)'

# Print the line number with along with each line
cat input.txt | kaw 'stdin.map((line, index) => `${index + 1} ${line}`)'

# Chain multiple transformations together
cat input.txt | kaw 'stdin.filter(line => line.length > 10).map((line, index) => `${index + 1} ${line}`)'
```

## Expressions

`kaw` accepts a single argument, a JavaScript expression that manipulates lines of stdin. `kaw` exposes a global variable `stdin` to the expression. `stdin` is an [`Iterable`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Iterators_and_generators#iterables) that yields one line of stdin at a time (with newline terminators stripped from the line). `kaw` uses V8 to execute the expressions, which supports [iterator helpers](https://v8.dev/features/iterator-helpers). This means that you can use methods like `.filter` or `.map` on `stdin` as if it were an array.

If the expression is iterable, `kaw` prints one line to stdout for each item. If the expression is not iterable, `kaw` simply prints the stringified result on a single line. `kaw` expressions can contain multiple lines, however the last line must evaluate to the result that should be written to stdout.

## Additional examples

```sh
# Print every line that starts with a #
cat input.txt | kaw 'stdin.filter(line => /^#/.test(line))'

# Replace foo with bar in every line
cat input.txt | kaw 'stdin.map(line => line.replaceAll("foo", "bar"))'

# Print 5 lines after the first 10 lines
cat input.txt | kaw 'stdin.drop(10).take(5)'

# Split each line into two lines
cat input.txt | kaw 'stdin.flatMap(line => [line.slice(0, Math.floor(line.length / 2)), line.slice(Math.floor(line.length / 2))])'

# Count the number of characters on all lines
cat input.txt | kaw 'stdin.reduce((total, line) => total + line.length, 0)'

# Reverse the lines
cat input.txt | kaw 'stdin.toArray().toReversed()'

# Deduplicate the lines
cat input.txt | kaw 'new Set(stdin).values()'

# Multiline expression (although at this point, you should probably just write a Deno or Node.js script)
cat input.txt | kaw 'const map = new Map(); stdin.forEach((line) => { map.set(line, (map.get(line) ?? 0) + 1) }); map.entries().toArray().toSorted(([line1, count1], [line2, count2]) => count2 - count1).map(([line, count]) => `${count} occurrences of ${line}`)'
```

## Performance

`kaw` is built in Rust and uses the `deno_core` crate, which powers deno. deno is already a fast and efficient JavaScript runtime, but by using `deno_core` directly and stripping out any unnecessary overhead, `kaw` achieves performance unmatched by deno or Node.js scripts. In the benchmark below, `kaw` is about 8 times faster than `awk`.

```sh
# Generate a 16MB text file with 64K lines of random data
base64 -i /dev/urandom | head -c 16777216 | fold -w 256 > input.txt

# Transform each line with kaw
time cat input.txt | kaw 'stdin.map(line => line.toUpperCase())' > output.txt

# Transform each line with awk
time cat input.txt | awk '{ print toupper($0) }' > output.txt

# Or use hyperfine for benchmarking
hyperfine --command-name kaw 'cat input.txt | kaw "stdin.map(line => line.toUpperCase())"' --command-name awk 'cat input.txt | awk "{ print toupper($0) }" > output.txt'
```

## License

`kaw` is distributed under the terms of the MIT License.
