use regex::Regex;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::ops::AddAssign;

const ADDRESS: &str = "127.0.0.1:7878";

fn form_html(content: &String) -> String {
    let status_line = "HTTP/1.1 200 OK";
    let len = content.len();
    let response = format!("{status_line}\r\nContent-Length: {len}\r\n\r\n{content}");
    response
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_req: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("REQEST: {:#?}", http_req);

    let uri = {
        let re = Regex::new(r"GET (.+) HTTP/1\.1").unwrap();

        let Some(method_line) = http_req.first() else { return; };
        let Some(caps) = re.captures(method_line) else { return; };
        let Some(cap) = caps.get(1) else { return; };
        cap.as_str()
    };

    println!("URI = {uri}");

    let entries: Option<Vec<_>> = {
        if let Ok(dir) = fs::read_dir(uri) {
            let mut vec = Vec::new();
            for d in dir {
                if let Ok(e) = d {
                    vec.push(e.path());
                }
            }
            Some(vec)
        } else {
            None
        }
    };

    {
        let mut info = format!("<p>path: {}</p>\n", uri);
        if let Some(entries) = entries {
            for e in entries {
                if let Some(e) = e.to_str() {
                    info.push_str(format!("<p>{}</p>\n", e).as_str());
                }
            }
        } else {
            info.push_str("<p>INVALID PATH!</p>");
        }

        let content = fs::read_to_string("data/hello.html").unwrap();
        let content = content.replace("$PLACEHOLDER$", &info);
        let response = form_html(&content);
        stream.write_all(response.as_bytes()).unwrap();
    }
}

fn run_server() -> Option<()> {
    let listener = TcpListener::bind(ADDRESS).unwrap();

    for stream in listener.incoming() {
        handle_connection(stream.unwrap());
    }

    Some(())
}

fn main() {
    if let None = run_server() {
        println!("SERVER PIZDES!");
    }
}