/*
    Copyright 2013 Jesse 'Jeaye' Wilkerson
    See licensing in LICENSE file, or at:
        http://www.opensource.org/licenses/BSD-3-Clause

    File: obj/md5/model.rs
    Author: Jesse 'Jeaye' Wilkerson
    Description:
      Loads, parses, and represents
      the model (as in MVC) side of
      MD5 animated models.
*/

use std::{ io, path, vec, i32 };
use super::{ Joint, Vertex, Triangle, Weight, Mesh };
use math;

struct Model
{
  version: i32,
  num_joints: i32,
  num_meshes: i32,
  is_animated: bool,

  joints: ~[Joint],
  meshes: ~[Mesh],

  //animation: Option<Animation>,
  
  local_to_world: math::Mat4x4,
}

impl Model
{
  pub fn new(mesh_file: ~str) -> Model
  {
    let mut model = Model
    {
      version: 0,
      num_joints: 0,
      num_meshes: 0,
      is_animated: false,

      joints: ~[],
      meshes: ~[],

      //animation: None,

      local_to_world: math::Mat4x4::new(),
    };

    model.load(mesh_file);

    model
  }

  priv fn load(&mut self, file: ~str) -> bool
  {
    let fior = io::file_reader(&path::Path(file));
    if fior.is_err()
    { error!("Failed to open model file %s", file); return false; }

    /* Clear existing data. */
    self.joints.clear();
    self.meshes.clear();

    let fio = fior.get();
    let mut param = ~"";
    let read_param = ||
    {
      param = ~""; /* TODO: clear? */
      let mut ch = fio.read_char();
      while ch.is_whitespace() && !fio.eof() /* Find the next word. */
      { ch = fio.read_char(); }

      if !fio.eof()
      { 
        param.push_char(ch);
        ch = fio.read_char();
        while !ch.is_whitespace() && !fio.eof() /* Read the next word. */
        { param.push_char(ch); ch = fio.read_char(); }
      }
    };
    macro_rules! read_type
    (
      ($var:expr) =>
      ({
        let name = param.clone();
        read_param();
        let num = FromStr::from_str(param);
        if num.is_some()
        { $var = num.get(); }
        else
        { error!("Invalid %s in %s", name, file); }
      });
    )

    /* Read the first param and jump into the parsing. */
    read_param();
    while !fio.eof()
    {
      match param
      {
        ~"MD5Version" =>
        {
          /* Read version. */
          read_type!(self.version);
          debug!("Model version: %?", self.version);
        }
        ~"commandline" =>
        { fio.read_line(); /* Ignore this line. */ }
        ~"numJoints" =>
        {
          read_type!(self.num_joints);
          self.joints = vec::with_capacity(self.num_joints as uint);
          debug!("Model joints: %?", self.num_joints);
        }
        ~"numMeshes" =>
        {
          read_type!(self.num_meshes);
          self.meshes = vec::with_capacity(self.num_meshes as uint);
          debug!("Model meshes: %?", self.num_meshes);
        }
        ~"joints" =>
        {
          let mut joint = Joint::new();
          read_param(); /* read { */
          debug!("Reading model joints");
          
          for i32::range(0, self.num_joints) |_|
          {
            read_param();
            joint.name = param.clone();

            read_type!(joint.parent);

            read_param(); /* junk */
            read_type!(joint.position.x);
            read_type!(joint.position.y);
            read_type!(joint.position.z);
            read_param(); /* junk */
            read_param(); /* junk */
            read_type!(joint.orientation.x);
            read_type!(joint.orientation.y);
            read_type!(joint.orientation.z);
            read_param(); /* junk */

            joint.orientation.compute_w();
            self.joints.push(joint.clone());

            /* Ignore the rest of the line. */
            fio.read_line();
          }

          read_param(); /* read } */
        }
        ~"mesh" =>
        {
          let mut mesh = Mesh::new();
          let mut vert = Vertex::new();
          let mut tri = Triangle::new();
          let mut weight = Weight::new();
          let mut num_verts = 0;
          let mut num_tris = 0;
          let mut num_weights = 0;

          debug!("Parsing mesh");

          read_param(); /* Read } */
          read_param();
          while param != ~"}"
          {
            match param
            {
              ~"shader" => /* shader == texture path */
              {
                read_param();
                mesh.texture = param.clone();
                debug!("Mesh shader: %s", mesh.texture);

                /* TODO: Load texture. */

                fio.read_line();
              }
              ~"numverts" =>
              {
                read_type!(num_verts);
                fio.read_line();

                debug!("Mesh verts: %?", num_verts);

                for i32::range(0, num_verts) |_|
                {
                  read_param(); /* junk */
                  read_param(); /* junk */
                  read_param(); /* junk */
                  read_type!(vert.tex_coord.x);
                  read_type!(vert.tex_coord.y);
                  read_param(); /* junk */
                  read_type!(vert.start_weight);
                  read_type!(vert.weight_count);

                  fio.read_line();

                  mesh.verts.push(vert);
                  mesh.tex_coords.push(vert.tex_coord);
                }
              }
              ~"numtris" =>
              {
                read_type!(num_tris);
                fio.read_line();
                debug!("Mesh tris: %?", num_tris);

                for i32::range(0, num_tris) |_|
                {
                  read_param(); /* junk */
                  read_param(); /* junk */
                  read_type!(tri.indices[0]);
                  read_type!(tri.indices[1]);
                  read_type!(tri.indices[2]);

                  fio.read_line();

                  mesh.triangles.push(tri);
                  mesh.indices.push(tri.indices[0] as u32);
                  mesh.indices.push(tri.indices[1] as u32);
                  mesh.indices.push(tri.indices[2] as u32);
                }
              }
              ~"numweights" =>
              {
                read_type!(num_weights);
                fio.read_line();
                debug!("Mesh weights: %?", num_weights);

                for i32::range(0, num_weights) |_|
                {
                  read_param(); /* junk */
                  read_param(); /* junk */
                  read_type!(weight.joint_id);
                  read_type!(weight.bias);
                  read_param(); /* junk */
                  read_type!(weight.position.x);
                  read_type!(weight.position.y);
                  read_type!(weight.position.z);
                  read_param(); /* junk */

                  fio.read_line();
                  mesh.weights.push(weight);
                }
              }
              _ =>
              { fio.read_line(); }
            }

            read_param();
          }

          /* TODO: Prepare mesh. */
          self.meshes.push(mesh);
        }
        _ => { } 
      }

      /* Move to the next param. */
      read_param();
    }

    true
  }
}

