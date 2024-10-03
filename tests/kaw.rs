use assert_cmd::Command;

static INPUT: &str = include_str!("./input.txt");

#[test]
fn test_noop() {
    let mut cmd = Command::cargo_bin("kaw").unwrap();
    cmd.arg("stdin")
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
        .stdout("[object Object]\n");
}
