use linear::Vector4F;
use linear::Vertex4F;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

struct Vertex {
    vi: usize,
    ni: usize,
    ti: usize,
}

//Loads triangles from an OBJ file. Only triangles are supported.
//In the returned vec, each pair of three values in a row form a triangle.
pub fn load_obj(filename: &str) -> Vec<Vertex4F> {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    let mut vertices: Vec<(f64, f64, f64)> = Vec::new();
    let mut normals: Vec<(f64, f64, f64)> = Vec::new();
    let mut tex_coords: Vec<(f64, f64)> = Vec::new();
    let mut faces: Vec<Vec<Vertex>> = Vec::new();

    for line in reader.lines() {
        if line.is_ok() {
            let l = line.unwrap();

            if l.starts_with("v ") {
                vertices.push(read_vertex(l));
            } else if l.starts_with("vt") {
                tex_coords.push(read_tex_coords(l));
            } else if l.starts_with("vn") {
                normals.push(read_normal(l));
            } else if l.starts_with("f") {
                faces.push(read_face(l));
            }
        }
    }

    let mut result = Vec::new();

    for face in faces {
        let mut has_normals = false;
        let mut verts = Vec::new();

        for v in face {
            let mut vert = Vertex4F::new();

            if v.vi > 0 {
                let vpos = vertices[v.vi - 1];
                vert.pos = Vector4F::new(vpos.0, vpos.1, vpos.2);
            }

            if v.ni > 0 {
                let vnorm = normals[v.ni - 1];
                vert.normal = Vector4F::new(vnorm.0, vnorm.1, vnorm.2).normalize();
                has_normals = true;
            }

            if v.ti > 0 {
                let vtex = tex_coords[v.ti - 1];
                vert.tex_u = vtex.0;
                vert.tex_v = vtex.1;
            }

            verts.push(vert);
        }

        if !has_normals {
            let edge1 = &verts[0].pos - &verts[1].pos;
            let edge2 = &verts[2].pos - &verts[1].pos;
            let cross = Vector4F::cross(&edge2, &edge1).normalize();

            for v in &mut verts {
                v.normal = cross.clone();
            }
        }

        for v in verts {
            result.push(v);
        }
    }

    result
}

fn read_vertex(line: String) -> (f64, f64, f64) {
    let tokens = split_line(&line);

    let x: f64 = tokens[1].parse().unwrap();
    let y: f64 = tokens[2].parse().unwrap();
    let z: f64 = tokens[3].parse().unwrap();

    (x, y, z)
}

fn read_tex_coords(line: String) -> (f64, f64) {
    let tokens = split_line(&line);

    let u: f64 = tokens[1].parse().unwrap();
    let v: f64 = tokens[2].parse().unwrap();

    (u, v)
}

fn read_normal(line: String) -> (f64, f64, f64) {
    read_vertex(line)
}

fn read_face(line: String) -> Vec<Vertex> {
    let tokens = split_line(&line);

    if tokens.len() != 4 {
        let mut message = String::new();
        message.push_str("Only faces with 3 vertices are supported! Line: ");
        message.push_str(line.as_str());
        panic!(message);
    }

    let v1 = read_face_vertex(&tokens[1]);
    let v2 = read_face_vertex(&tokens[2]);
    let v3 = read_face_vertex(&tokens[3]);

    vec![v1, v2, v3]
}

fn read_face_vertex(token: &String) -> Vertex {
    let parts: Vec<&str> = token.split('/').collect();

    if parts.len() <= 0 || parts.len() > 3 {
        let mut message = String::new();
        message.push_str("Value is not a valid face: ");
        message.push_str(token.as_str());
        panic!(message);
    }

    let mut vi: usize = 0;
    let mut ni: usize = 0;
    let mut ti: usize = 0;

    if parts.len() >= 1 {
        vi = parts[0].parse().unwrap();
    }
    if parts.len() >= 2 {
        if parts[1].len() > 0 {
            ti = parts[1].parse().unwrap();
        }
    }
    if parts.len() >= 3 {
        if parts[2].len() > 0 {
            ni = parts[2].parse().unwrap();
        }
    }

    Vertex { vi, ni, ti }
}

fn split_line(line: &String) -> Vec<String> {
    let mut result = Vec::with_capacity(4);

    for token in line.split_whitespace() {
        result.push(String::from(token));
    }

    result
}
