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

let writeLineOp = null;

const writeLine = (value) => {
  if (value !== null && value !== undefined) {
    writeLineOp(`${value}`);
  }
};

const isIterable = (result) => {
  if (result === null || typeof result !== 'object' || result instanceof String) {
    // Don't iterate strings
    return false;
  }
  const method = result[Symbol.iterator];
  if (typeof method === 'function') {
    return true;
  }
  if (method === null || method === undefined) {
    return false;
  }
  throw new TypeError(`${Deno.core.ops.op_detail_string(result)} is not iterable`);
};

Object.defineProperty(globalThis, '__kawWrite', {
  value: (result) => {
    writeLineOp ||= Deno.core.ops.op_write_line;

    // Fast path for arrays because index iteration is faster than for-of loop
    if (Array.isArray(result)) {
      for (let index = 0; index < result.length; ++index) {
        writeLine(result[index]);
      }
      return;
    }

    if (!isIterable(result)) {
      writeLine(result);
      return;
    }

    for (const value of result) {
      writeLine(value);
    }
  }
});
