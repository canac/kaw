use assert_cmd::{Command, cargo::cargo_bin};
use std::process::{Command as StdCommand, Stdio};

static INPUT: &str = include_str!("./input.txt");
static UPPERCASE: &str = "stdin.map(line => line.toUpperCase())";

fn assert_error(expression: &str, message: &str) {
    let mut cmd = Command::cargo_bin("kaw").unwrap();
    let output = cmd.arg(expression).write_stdin(INPUT).output().unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).starts_with(message));
}

#[test]
fn test_noop() {
    let mut cmd = Command::cargo_bin("kaw").unwrap();
    cmd.arg("stdin")
        .write_stdin(INPUT)
        .assert()
        .stdout("Line 1\nLine 2\nLine 3\nLine A\nLine B\nLine C\n");
}

#[test]
fn test_crlf() {
    let mut cmd = Command::cargo_bin("kaw").unwrap();
    cmd.arg("stdin")
        .write_stdin("Line 1\r\nLine 2\r\nLine 3\r\nLine A\r\nLine B\r\nLine C\r\n")
        .assert()
        .stdout("Line 1\nLine 2\nLine 3\nLine A\nLine B\nLine C\n");
}

#[test]
fn test_invalid_utf8() {
    let mut cmd = Command::cargo_bin("kaw").unwrap();
    let output = cmd
        .arg("stdin")
        .write_stdin(b"Line 1\n\xff\n".to_vec())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(output.stdout, b"Line 1\n");
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .starts_with("Error: Error: Failed to read stdin: stream did not contain valid UTF-8")
    );
}

#[test]
fn test_abbreviation() {
    let mut cmd = Command::cargo_bin("kaw").unwrap();
    cmd.arg("s")
        .write_stdin(INPUT)
        .assert()
        .stdout("Line 1\nLine 2\nLine 3\nLine A\nLine B\nLine C\n");
}

#[test]
fn test_filter() {
    let mut cmd = Command::cargo_bin("kaw").unwrap();
    cmd.arg("stdin.filter(line => line.endsWith(3) || line === 'Line C')")
        .write_stdin(INPUT)
        .assert()
        .stdout("Line 3\nLine C\n");
}

#[test]
fn test_map() {
    let mut cmd = Command::cargo_bin("kaw").unwrap();
    cmd.arg("stdin.map(line => line.slice(5))")
        .write_stdin(INPUT)
        .assert()
        .stdout("1\n2\n3\nA\nB\nC\n");
}

#[test]
fn test_take() {
    let mut cmd = Command::cargo_bin("kaw").unwrap();
    cmd.arg("stdin.take(2)")
        .write_stdin(INPUT)
        .assert()
        .stdout("Line 1\nLine 2\n");
}

#[test]
fn test_drop() {
    let mut cmd = Command::cargo_bin("kaw").unwrap();
    cmd.arg("stdin.drop(4)")
        .write_stdin(INPUT)
        .assert()
        .stdout("Line B\nLine C\n");
}

#[test]
fn test_null_undefined() {
    let mut cmd = Command::cargo_bin("kaw").unwrap();
    cmd.arg("stdin.map((line, index) => index === 0 ? null : index === 1 ? undefined : line)")
        .write_stdin(INPUT)
        .assert()
        .stdout("Line 3\nLine A\nLine B\nLine C\n");
}

#[test]
fn test_array() {
    let mut cmd = Command::cargo_bin("kaw").unwrap();
    cmd.arg("['Line 1', null, 2, undefined]")
        .write_stdin(INPUT)
        .assert()
        .stdout("Line 1\n2\n");
}

#[test]
fn test_non_array() {
    let mut cmd = Command::cargo_bin("kaw").unwrap();
    cmd.arg("({ key: 'value' })")
        .write_stdin(INPUT)
        .assert()
        .success()
        .stdout("[object Object]\n");
}

#[test]
fn test_set() {
    let mut cmd = Command::cargo_bin("kaw").unwrap();
    cmd.arg("new Set(['Line 1', 'Line 2'])")
        .write_stdin(INPUT)
        .assert()
        .stdout("Line 1\nLine 2\n");
}

#[test]
fn test_map_entries() {
    let mut cmd = Command::cargo_bin("kaw").unwrap();
    cmd.arg("new Map([['a', 1], ['b', 2]])")
        .write_stdin(INPUT)
        .assert()
        .stdout("a,1\nb,2\n");
}

#[test]
fn test_boxed_string() {
    let mut cmd = Command::cargo_bin("kaw").unwrap();
    cmd.arg("new String('Line 1')")
        .write_stdin(INPUT)
        .assert()
        .stdout("Line 1\n");
}

#[test]
fn test_custom_iterable() {
    let mut cmd = Command::cargo_bin("kaw").unwrap();
    cmd.arg("({ *[Symbol.iterator]() { yield 'Line 1'; yield 'Line 2' } })")
        .write_stdin(INPUT)
        .assert()
        .stdout("Line 1\nLine 2\n");
}

#[test]
fn test_throwing_callback() {
    assert_error(
        "stdin.map(line => x)",
        "Error: ReferenceError: x is not defined",
    );
}

#[test]
fn test_throwing_next_getter() {
    assert_error(
        "({ [Symbol.iterator]: () => ({ get next() { throw new Error('getter') } }) })",
        "Error: Error: getter",
    );
}

#[test]
fn test_throwing_array_element() {
    assert_error(
        "Object.defineProperty(['a'], 0, { get() { throw new Error('element') } })",
        "Error: Error: element",
    );
}

#[test]
fn test_throwing_done_getter() {
    assert_error(
        "({ [Symbol.iterator]: () => ({ next: () => ({ get done() { \
          throw new Error('done') } }) }) })",
        "Error: Error: done",
    );
}

#[test]
fn test_throwing_value_getter() {
    assert_error(
        "({ [Symbol.iterator]: () => ({ next: () => ({ get value() { \
          throw new Error('value') } }) }) })",
        "Error: Error: value",
    );
}

#[test]
fn test_unstringifiable_array_element() {
    assert_error(
        "[Symbol('a')]",
        "Error: TypeError: Cannot convert a Symbol value to a string",
    );
}

#[test]
fn test_unstringifiable_iterator_value() {
    assert_error(
        "new Set([Symbol('a')])",
        "Error: TypeError: Cannot convert a Symbol value to a string",
    );
}

#[test]
fn test_unstringifiable_result() {
    assert_error(
        "Symbol('a')",
        "Error: TypeError: Cannot convert a Symbol value to a string",
    );
}

#[test]
fn test_malformed_iterator() {
    assert_error(
        "({ [Symbol.iterator]: () => ({ next: () => 0 }) })",
        "Error: TypeError: Iterator result 0 is not an object",
    );
}

#[test]
fn test_malformed_iterable() {
    assert_error(
        "({ [Symbol.iterator]() { return 0 } })",
        "Error: TypeError: Result of the Symbol.iterator method is not an object",
    );
}

#[test]
fn test_non_callable_iterable() {
    assert_error(
        "({ [Symbol.iterator]: 5, next: () => ({ value: 'Line 1', done: false }) })",
        "Error: TypeError: #<Object> is not iterable",
    );
}

#[test]
fn test_no_args() {
    let mut cmd = Command::cargo_bin("kaw").unwrap();
    cmd.assert().stderr("Usage: kaw [expression] [args...]\n");
}

#[test]
fn test_arg() {
    let mut cmd = Command::cargo_bin("kaw").unwrap();
    cmd.args(["args", "1", "2", "3"])
        .assert()
        .stdout("1\n2\n3\n");
}

#[test]
fn test_large_input() {
    let input: String = (0..50_000).map(|i| format!("Line {i}\n")).collect();
    let expected: String = (0..50_000).map(|i| format!("{i}\n")).collect();
    let mut cmd = Command::cargo_bin("kaw").unwrap();
    cmd.arg("stdin.map(line => line.slice(5))")
        .write_stdin(input)
        .assert()
        .stdout(expected);
}

#[test]
fn test_broken_pipe() {
    use std::io::pipe;

    // Dropping the read end makes every write to stdout fail
    let (reader, writer) = pipe().unwrap();
    drop(reader);

    let output = StdCommand::new(cargo_bin("kaw"))
        // Enough lines to overflow the output buffer and reach the closed pipe
        .arg("Array.from({ length: 2000 }, (_, index) => `Line ${index}`)")
        .stdin(Stdio::null())
        .stdout(writer)
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
}

fn assert_streams_first_line(expression: &str, input: &str) {
    use std::io::{BufRead, BufReader, Write};
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    let mut child = StdCommand::new(cargo_bin("kaw"))
        .arg(expression)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    // Feed the input but keep stdin open
    let mut stdin = child.stdin.take().unwrap();
    write!(stdin, "{input}").unwrap();
    stdin.flush().unwrap();

    // Read a line without blocking
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut line = String::new();
        stdout.read_line(&mut line).unwrap();
        let _ = tx.send(line);
    });

    // Check the output for the transformed line before stdin closes
    let line = rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(line, "FIRST\n");

    drop(stdin);
    child.wait().unwrap();
}

#[test]
fn test_streams_before_input_ends() {
    assert_streams_first_line(UPPERCASE, "first\n");
}

#[test]
fn test_streams_before_line_ends() {
    assert_streams_first_line(UPPERCASE, "first\npartial");
}

#[test]
fn test_streams_when_a_later_line_writes_nothing() {
    assert_streams_first_line(
        "stdin.filter(line => line === 'first').map(line => line.toUpperCase())",
        "first\nsecond\n",
    );
}
