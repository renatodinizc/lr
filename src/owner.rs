#[derive(Clone, Copy)]
enum Owner {
    User,
    Group,
    Other,
}

impl Owner {
    fn masks(&self) -> [u32; 3] {
        match self {
            Self::User => [0o400, 0o200, 0o100],
            Self::Group => [0o040, 0o020, 0o010],
            Self::Other => [0o004, 0o002, 0o001],
        }
    }
}

pub fn format_mode(mode: u32) -> String {
    format!(
        "{}{}{}",
        mk_triple(mode, Owner::User),
        mk_triple(mode, Owner::Group),
        mk_triple(mode, Owner::Other),
    )
}

fn mk_triple(mode: u32, owner: Owner) -> String {
    let [read, write, execute] = owner.masks();
    format!(
        "{}{}{}",
        if mode & read == 0 { "-" } else { "r" },
        if mode & write == 0 { "-" } else { "w" },
        if mode & execute == 0 { "-" } else { "x" },
    )
}
