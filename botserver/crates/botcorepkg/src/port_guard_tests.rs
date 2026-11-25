use super::*;

fn words(values: &[u32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_ne_bytes()).collect()
}

fn net_port(port: u16) -> u32 {
    u32::from(port.to_be())
}

#[test]
fn ipv4_listener_table_yields_ports_and_pids() {
    let buffer = words(&[
        2,
        2, 0x0100_007f, net_port(5432), 0, 0, 4242,
        2, 0, net_port(8300), 0, 0, 77,
    ]);
    assert_eq!(
        listeners_from_table(&buffer, false),
        vec![Listener { port: 5432, pid: 4242 }, Listener { port: 8300, pid: 77 }]
    );
}

#[test]
fn ipv6_listener_table_yields_ports_and_pids() {
    let mut row = vec![0u32; 14];
    row[5] = net_port(8201);
    row[13] = 999;
    let mut values = vec![1];
    values.extend(row);
    assert_eq!(listeners_from_table(&words(&values), true), vec![Listener { port: 8201, pid: 999 }]);
}

#[test]
fn truncated_table_is_not_read_past_its_end() {
    let buffer = words(&[3, 2, 0, net_port(80), 0, 0, 1]);
    assert_eq!(listeners_from_table(&buffer, false), vec![Listener { port: 80, pid: 1 }]);
    assert!(listeners_from_table(&[], false).is_empty());
}

#[test]
fn stack_programs_are_recognised_without_regard_to_case() {
    let stack = Path::new(r"C:\Users\a\AppData\Local\generalbots\stack");
    let ours = PathBuf::from(r"c:\users\a\appdata\local\GeneralBots\stack\bin\vault\vault.exe");
    let other_user = PathBuf::from(r"C:\Users\b\AppData\Local\generalbots\stack\bin\vault\vault.exe");
    let sibling = PathBuf::from(r"C:\Users\a\AppData\Local\generalbots\stack-old\bin\vault.exe");
    assert!(belongs_to_stack(Some(&ours), stack));
    assert!(!belongs_to_stack(Some(&other_user), stack));
    assert!(!belongs_to_stack(Some(&sibling), stack));
    assert!(!belongs_to_stack(None, stack));
}

#[test]
fn conflict_message_names_port_service_and_program() {
    let message = conflict_message("tables", 5432, Some(Path::new(r"C:\Program Files\PostgreSQL\16\bin\postgres.exe")));
    assert!(message.contains("5432"));
    assert!(message.contains("\"tables\""));
    assert!(message.contains("postgres.exe"));
    assert!(message.contains("já está em uso"));
    assert!(conflict_message("cache", 6379, None).contains("outro programa"));
}

#[test]
fn free_ports_pass() {
    assert!(ensure_ports_free("test", &[], Path::new("/nonexistent")).is_ok());
}
