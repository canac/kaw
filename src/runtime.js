const lines = function* () {
  while (true) {
    const line = Deno.core.ops.op_stdin_line();
    if (line === null) {
      return;
    }
    yield line;
  }
};


globalThis.stdin = globalThis.s = lines();
