use crate::protocol;

#[derive(Clone, Copy)]
struct Security {
    security_id: u64,
    description: &'static str,
}

const SECURITIES: &'static [Security] = &[
    Security {
        security_id: 0,
        description: "first security",
    },
    Security {
        security_id: 1,
        description: "second security",
    },
    Security {
        security_id: 2,
        description: "third security",
    },
];

pub(crate) fn map_vec<R>(f: impl Fn(protocol::Security) -> R) -> Vec<R> {
    SECURITIES.iter().map(Into::into).map(f).collect()
}
pub(crate) fn as_vec() -> Vec<protocol::Security> {
    SECURITIES.iter().map(Into::into).collect()
}

impl From<&Security> for protocol::Security {
    fn from(rhs: &Security) -> Self {
        Self {
            security_id: rhs.security_id,
            description: rhs.description.into(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{map_vec, SECURITIES};

    #[test]
    fn numbered() {
        let len = SECURITIES.len() as u64;
        for (id, expected) in map_vec(|s| s.security_id).into_iter().zip(0..len) {
            assert_eq!(id, expected)
        }
    }
}
