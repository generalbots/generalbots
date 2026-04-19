
use std::collections::HashSet;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Mutex;

static PORT_COUNTER: AtomicU16 = AtomicU16::new(15000);
static ALLOCATED_PORTS: Mutex<Option<HashSet<u16>>> = Mutex::new(None);

pub struct PortAllocator;

impl PortAllocator {
    pub fn allocate() -> u16 {
        loop {
            let port = PORT_COUNTER.fetch_add(1, Ordering::SeqCst);
            if port > 60000 {
                PORT_COUNTER.store(15000, Ordering::SeqCst);
                continue;
            }

            if Self::is_available(port) {
                let mut guard = ALLOCATED_PORTS.lock().unwrap();
                let set = guard.get_or_insert_with(HashSet::new);
                set.insert(port);
                return port;
            }
        }
    }

    #[must_use]
    pub fn allocate_range(count: usize) -> Vec<u16> {
        (0..count).map(|_| Self::allocate()).collect()
    }

    pub fn release(port: u16) {
        let mut guard = ALLOCATED_PORTS.lock().unwrap();
        if let Some(set) = guard.as_mut() {
            set.remove(&port);
        }
    }

    fn is_available(port: u16) -> bool {
        use std::net::TcpListener;
        TcpListener::bind(("127.0.0.1", port)).is_ok()
    }
}

#[derive(Debug)]
pub struct TestPorts {
    pub postgres: u16,
    pub minio: u16,
    pub redis: u16,
    pub botserver: u16,
    pub mock_zitadel: u16,
    pub mock_llm: u16,
}

impl TestPorts {
    pub fn allocate() -> Self {
        Self {
            postgres: PortAllocator::allocate(),
            minio: PortAllocator::allocate(),
            redis: PortAllocator::allocate(),
            botserver: PortAllocator::allocate(),
            mock_zitadel: PortAllocator::allocate(),
            mock_llm: PortAllocator::allocate(),
        }
    }
}

impl Drop for TestPorts {
    fn drop(&mut self) {
        if self.postgres >= 15000 {
            PortAllocator::release(self.postgres);
        }
        if self.minio >= 15000 {
            PortAllocator::release(self.minio);
        }
        if self.redis >= 15000 {
            PortAllocator::release(self.redis);
        }
        if self.botserver >= 15000 {
            PortAllocator::release(self.botserver);
        }
        if self.mock_zitadel >= 15000 {
            PortAllocator::release(self.mock_zitadel);
        }
        if self.mock_llm >= 15000 {
            PortAllocator::release(self.mock_llm);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_allocation() {
        let port1 = PortAllocator::allocate();
        let port2 = PortAllocator::allocate();
        assert_ne!(port1, port2);
        assert!(port1 >= 15000);
        assert!(port2 >= 15000);
    }

    #[test]
    fn test_ports_struct() {
        let ports = TestPorts::allocate();
        assert_ne!(ports.postgres, ports.minio);
        assert_ne!(ports.redis, ports.botserver);
    }
}
