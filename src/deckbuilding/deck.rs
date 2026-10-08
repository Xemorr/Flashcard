use std::path::Path;

use hyperflash::deck::{
    Deck, ManagedDeck,
    model::{
        Field, Model,
        catalog::ModelCatalog,
    },
    note::ids::GitNoteIdGenerator,
};
use hyperflash::error::DeckError;
use uuid::Uuid;

const BASIC_MODEL_ID: &str = "826451dc-dd52-4710-a416-d0e0f9109480";

/// A [`ModelCatalog`] that hands back this app's built-in [`basic_model`] instead
/// of reading `.model` directories off disk. hyperflash's default
/// `FilesystemModelCatalog` only looks in `$XDG_DATA_HOME/flash/*.model`, which
/// has nothing registered for our app-defined "Basic" model, so `Deck::from`
/// (which always uses `FilesystemModelCatalog`) fails to resolve any deck whose
/// `index.flash` references `/ Basic /`. Using `Deck::from_with` with this
/// catalog keeps the model definition co-located with the code that builds
/// `Note`s from it (`basic_model()`), with no on-disk registration step needed.
struct BasicModelCatalog;

impl ModelCatalog for BasicModelCatalog {
    fn load_models(&self, _deck_path: &Path) -> Result<Vec<Model>, DeckError> {
        Ok(vec![basic_model()])
    }
}

pub fn open_or_init_deck(path: &str) -> ManagedDeck {
    if Path::new(path).join(".git").exists() {
        Deck::from_with(path, &BasicModelCatalog, &GitNoteIdGenerator)
            .unwrap_or_else(|err| panic!("failed to open existing deck at {path}: {err}"))
            .managed()
            .expect("deck has no working directory")
    } else {
        Deck::init(path).unwrap_or_else(|err| panic!("failed to create deck at {path}: {err}"))
    }
}

pub fn basic_model() -> Model {
    Model {
        name: "Basic".to_string(),
        id: Uuid::parse_str(BASIC_MODEL_ID).unwrap(),
        fields: vec![
            Field {
                name: "Front".into(),
                sticky: None,
                associated_media: None,
            },
            Field {
                name: "Back".into(),
                sticky: None,
                associated_media: None,
            },
        ],
        ..Model::default()
    }
}
