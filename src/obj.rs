use std::cmp::Eq;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use linear::Vector4F;
use linear::Vertex4F;

struct Vertex {
  vi: usize,
  ni: usize,
  ti: usize,
}

impl PartialEq for Vertex {
  fn eq(&self, other: &Vertex) -> bool {
    return self.vi == other.vi &&
           self.ni == other.ni &&
           self.ti == other.ti;
  }
}

impl Eq for Vertex {}

impl Hash for Vertex {
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_usize(self.vi);
    state.write_usize(self.ni);
    state.write_usize(self.ti);
  }
}

//Loads triangles from an OBJ file. Only triangles are supported.
//In the returned tuple, the first value is the list of unique vertices for the mesh.
//The second value is a list of 3-tuples with vertex indexes. Each tuple is a triangle.
pub fn load_obj(filename: &str) -> (Vec<Vertex4F>, Vec<(usize, usize, usize)>) {
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

  //Map from OBJ vertex to mesh vertex index
  let mut vertex_map: HashMap<Vertex, usize> = HashMap::new();
  //List of mesh vertices
  let mut mesh_vertices = Vec::new();
  //List of 3-tuples of mesh vertex indexes, forming the triangles
  let mut mesh_face_indexes = Vec::new();

  for face in faces {
    //TODO: Get rid of this? Maybe fixed size array or even directly tuple?
    let mut face_indexes = Vec::with_capacity(3);
    let mut has_normals = false;
   
    for v in face {
    	if vertex_map.contains_key(&v) {
      	let index = vertex_map.get(&v).unwrap();
      	face_indexes.push(*index);
    	}
    	else {
      	let mut vertex = Vertex4F::new();
      	
        if v.vi > 0 {
          let vpos = vertices[v.vi - 1];
          vertex.pos = Vector4F::new(vpos.0, vpos.1, vpos.2);
        }

        if v.ni > 0 {
          let vnorm = normals[v.ni - 1];
          vertex.normal = Vector4F::new(vnorm.0, vnorm.1, vnorm.2).normalize();
          has_normals = true;
        }

        if v.ti > 0 {
          let vtex = tex_coords[v.ti - 1];
          vertex.tex_u = vtex.0;
          vertex.tex_v = vtex.1;
        }
     	 
      	let index = mesh_vertices.len();
      	mesh_vertices.push(vertex);
      	face_indexes.push(index);
      	vertex_map.insert(v, index);
    	}
    }

    if !has_normals {
      let i0 = face_indexes[0];
      let i1 = face_indexes[1];
      let i2 = face_indexes[2];

      let mut vert0 = mesh_vertices[i0].clone();
      let mut vert1 = mesh_vertices[i1].clone();
      let mut vert2 = mesh_vertices[i2].clone();

      let edge1 = &vert0.pos - &vert1.pos;
      let edge2 = &vert2.pos - &vert1.pos;
      let cross = Vector4F::cross(&edge2, &edge1).normalize();

      vert0.normal = cross.clone();
      vert1.normal = cross.clone();
      vert2.normal = cross;

      mesh_vertices[i0] = vert0;
      mesh_vertices[i1] = vert1;
      mesh_vertices[i2] = vert2;
    }
   
    mesh_face_indexes.push((face_indexes[0], face_indexes[1], face_indexes[2]));
  }

  return (mesh_vertices, mesh_face_indexes);

  /*
  let mesh = Mesh::new();
  mesh.vertices = meshVertices;
  mesh.faces = meshFaceIndexes;
  */

  /*
  for face in faces {
    let mut has_normals = false;
    let mut verts = Vec::new();

    for v in face {
      let mut vertIndex;

      let vertIndex = vertexMap.get(&v);
      if vertIndex.is_some() {
        meshVertexIndexes.push(*vertIndex.unwrap());
      }
      else {
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

        meshVertices.push(vert);
        let newIndex = meshVertices.len() - 1;
        meshVertexIndexes.push(newIndex);
        vertexMap.insert(v, newIndex);
      }
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
  */
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
