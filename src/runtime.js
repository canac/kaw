class StdinIterator extends Iterator {
  next() {
    const line = Deno.core.ops.op_stdin_line();
    if (line === null) {
      return { done: true };
    }
    return { value: line, done: false };
  }
}

globalThis.stdin = new StdinIterator();
