const lines = function* () {
  while (true) {
    const line = Deno.core.ops.op_stdin_line();
    if (line === null) {
      return;
    }
    yield line;
  }
};

let args = null;
Object.defineProperty(globalThis, 'args', {
  get() {
    // Call op_args only once and cache the result
    args ||= Deno.core.ops.op_args();
    return args;
  }
});

globalThis.stdin = globalThis.s = lines();
