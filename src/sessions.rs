
struct Session {
    id: String,
    name: String,
    description: String,
    command: String,
}

struct Sessions {
    sessions: Vec<Session>,
}

impl Sessions {
    fn new() -> Self {
        // Sessions {
        //     sessions: Vec::new(),
        // }

        let path = "/usr/share/wayland-sessions/";
        let mut sessions = Vec::new();
        let entries = std::fs::read_dir(path).unwrap();
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().unwrap() == "desktop" {
                let file = std::fs::File::open(path).unwrap();
                let reader = std::io::BufReader::new(file);
                let mut session = Session {
                    id: String::new(),
                    name: String::new(),
                    description: String::new(),
                    command: String::new(),
                };
                for line in reader.lines() {
                    let line = line.unwrap();
                    if line.starts_with("Name=") {
                        session.name = line[5..].to_string();
                    } else if line.starts_with("Comment=") {
                        session.description = line[8..].to_string();
                    } else if line.starts_with("Exec=") {
                        session.command = line[5..].to_string();
                    }
                }
                sessions.push(session);
            }
        }

        Sessions { sessions }
    }

}
