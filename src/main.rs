mod thread_pool;
use regex::Regex;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use thread_pool::ThreadPool;

const ADDRESS: &str = "127.0.0.1:7878";

fn form_html(content: &String) -> String {
    let status_line = "HTTP/1.1 200 OK";
    let len = content.len();
    let response = format!("{status_line}\r\nContent-Length: {len}\r\n\r\n{content}");
    response
}

enum Entry {
    Dir(String),
    File(String),
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_req: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let uri = {
        let re = Regex::new(r"GET (.+) HTTP/1\.1").unwrap();

        let Some(method_line) = http_req.first() else { return; };
        let Some(caps) = re.captures(method_line) else { return; };
        let Some(cap) = caps.get(1) else { return; };
        cap.as_str()
    };

    let entries: Option<Vec<Entry>> = {
        if let Ok(dir) = fs::read_dir(uri) {
            let mut vec = Vec::new();
            for d in dir {
                if let Ok(e) = d {
                    let meta = e.metadata().unwrap();
                    let path = e.path().as_path().as_os_str().to_str().unwrap().to_string();
                    if meta.is_dir() || meta.is_symlink() {
                        vec.push(Entry::Dir(path));
                    } else if meta.is_file() {
                        vec.push(Entry::File(path));
                    }
                }
            }
            Some(vec)
        } else {
            None
        }
    };

    {
        let mut info = format!("<h1>path: {}</h1>\n", uri);
        let spamval = {
            const XXX: usize = 0xFF_FF_FF * 5;
            let mut spamvec = Vec::with_capacity(XXX);
            for _ in 0..XXX {
                spamvec.push(1);
            }
            spamvec.iter().fold(0, |a, sum| sum + a)
        };

        info.push_str(format!("<p>spam= {}</p>", spamval).as_str());
        info.push_str(format!("<p>thread id = {}</p>\n", unsafe { libc::gettid() }).as_str());

        {
            let parent = (|| -> Option<&str> {
                let path = Path::new(uri);
                Some(path.parent()?.to_str()?)
            })();

            if let Some(p) = parent {
                info.push_str(format!("<h2><a href=\"{}\">BACK TO {}</a></h2>", p, p).as_str());
            }
        }

        match entries {
            Some(entries) => {
                info.push_str("<ul>");
                for e in entries {
                    let entry = match e {
                        Entry::File(f) => {
                            format!("<li><a href=\"{}\">{}</a></li>", f, f)
                        }
                        Entry::Dir(d) => {
                            format!("<li><b><a href=\"{}\">{}</a></b></li>", d, d)
                        }
                    };
                    info.push_str(entry.as_str());
                }
                info.push_str("</ul>");
            }
            None => {
                info.push_str("<p>INVALID PATH!</p>");
            }
        }

        let content = fs::read_to_string("data/hello.html").unwrap();
        let content = content.replace("$PLACEHOLDER$", &info);
        let response = form_html(&content);
        stream.write_all(response.as_bytes()).unwrap();
    }
}

fn run_server() -> Option<()> {
    let listener = TcpListener::bind(ADDRESS).unwrap();
    let mut pool = ThreadPool::new(2);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.run(|| {
            handle_connection(stream);
        });
    }

    Some(())
}

fn main() {
    if let None = run_server() {
        println!("SERVER PIZDES!");
    }
}
