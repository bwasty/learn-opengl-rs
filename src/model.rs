#![allow(dead_code)] // TODO!! TMP

use std::path::Path;

use obj;

use mesh::{ Mesh, Texture };

#[derive(Default)]
pub struct Model {
    /*  Model Data */
    textures_loaded: Vec<Texture>,   // stores all the textures loaded so far, optimization to make sure textures aren't loaded more than once.
    pub meshes: Vec<Mesh>,
    directory: String,
    gammaCorrection: bool,
}

impl Model {
    /// constructor, expects a filepath to a 3D model.
    pub fn new(path: &str, gamma: bool) -> Model {
        let mut model = Model { gammaCorrection: gamma, ..Model::default() };
        model.loadModel(path);
        model
    }

    // loads a model from file and stores the resulting meshes in the meshes vector.
    fn loadModel(&mut self, path: &str) {
        let path = Path::new(path);
        let result = obj::load::<obj::SimplePolygon>(path);
        // retrieve the directory path of the filepath
        self.directory = path.parent().unwrap_or(Path::new("")).to_str().unwrap().into();

        // TODO!!!
    }
}
