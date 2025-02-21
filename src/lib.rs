use std::path::Path;

use crate::core::register_shard;
use crate::shard::Shard;
use bg3_lib::lsf_reader::Resource;
use bg3_lib::package::Package;
use bg3_lib::package_reader::PackageReader;
use ctor::*;
use lazy_static::lazy_static;
use shards::types::{
    AutoSeqVar, AutoTableVar, ClonedVar, Context, ExposedTypes, InstanceData, TableVar, Type,
    Types, Var, ANY_TABLE_TYPES, NONE_TYPES, STRING_TYPES,
};
use shards::*;

lazy_static! {
    pub static ref PACKAGE_TYPE: Type =
        Type::object(fourCharacterCode(*b"BG3 "), fourCharacterCode(*b"PKG "));
    pub static ref PACKAGE_TYPES: Vec<Type> = vec![*PACKAGE_TYPE];
    pub static ref RESOURCE_TYPE: Type =
        Type::object(fourCharacterCode(*b"BG3 "), fourCharacterCode(*b"RES "));
    pub static ref RESOURCE_TYPES: Vec<Type> = vec![*RESOURCE_TYPE];
}

pub mod larian_save_file {
    use super::*;

    pub struct LarianSaveFile(pub PackageReader, pub Package);
    ref_counted_object_type_impl!(LarianSaveFile);
}

pub mod larian_resource {
    use super::*;

    pub struct LarianResource(pub Resource);
    ref_counted_object_type_impl!(LarianResource);
}

#[derive(shards::shard)]
#[shard_info("BG3.LoadSaveFile", "Loads a save file")]
struct BG3LoadSaveFile {
    #[shard_required]
    required: ExposedTypes,
    package: ClonedVar,
}

impl Default for BG3LoadSaveFile {
    fn default() -> Self {
        Self {
            required: ExposedTypes::new(),
            package: ClonedVar::default(),
        }
    }
}

#[shards::shard_impl]
impl Shard for BG3LoadSaveFile {
    fn input_types(&mut self) -> &Types {
        &STRING_TYPES
    }

    fn output_types(&mut self) -> &Types {
        &PACKAGE_TYPES
    }

    fn warmup(&mut self, ctx: &Context) -> Result<(), &str> {
        self.warmup_helper(ctx)?;

        Ok(())
    }

    fn cleanup(&mut self, ctx: Option<&Context>) -> Result<(), &str> {
        self.cleanup_helper(ctx)?;

        Ok(())
    }

    fn compose(&mut self, data: &InstanceData) -> Result<Type, &str> {
        self.compose_helper(data)?;
        Ok(self.output_types()[0])
    }

    fn activate(&mut self, _context: &Context, input: &Var) -> Result<Option<SHVar>, &str> {
        let path: &str = input.try_into()?;
        let path = Path::new(path);
        let mut reader = PackageReader::new(path).map_err(|e| {
            shlog_error!("Failed to open package: {}", e);
            "Failed to open package"
        })?;
        let package = reader.read().map_err(|e| {
            shlog_error!("Failed to read package: {}", e);
            "Failed to read package"
        })?;

        self.package = Var::new_ref_counted(
            larian_save_file::LarianSaveFile(reader, package),
            &PACKAGE_TYPE,
        )
        .into();
        Ok(Some(self.package.0))
    }
}

#[derive(shards::shard)]
#[shard_info("BG3.Globals", "Gets the globals resource from a BG3 package")]
struct BG3Globals {
    #[shard_required]
    required: ExposedTypes,
    output: AutoTableVar,
}

impl Default for BG3Globals {
    fn default() -> Self {
        Self {
            required: ExposedTypes::new(),
            output: AutoTableVar::new(),
        }
    }
}

#[shards::shard_impl]
impl Shard for BG3Globals {
    fn input_types(&mut self) -> &Types {
        &PACKAGE_TYPES
    }

    fn output_types(&mut self) -> &Types {
        &ANY_TABLE_TYPES
    }

    fn warmup(&mut self, ctx: &Context) -> Result<(), &str> {
        self.warmup_helper(ctx)?;
        Ok(())
    }

    fn cleanup(&mut self, ctx: Option<&Context>) -> Result<(), &str> {
        self.cleanup_helper(ctx)?;
        Ok(())
    }

    fn compose(&mut self, data: &InstanceData) -> Result<Type, &str> {
        self.compose_helper(data)?;
        Ok(self.output_types()[0])
    }

    fn activate(&mut self, _context: &Context, input: &Var) -> Result<Option<SHVar>, &str> {
        let save_file = unsafe {
            &mut *Var::from_ref_counted_object::<larian_save_file::LarianSaveFile>(
                input,
                &*PACKAGE_TYPE,
            )?
        };

        let globals = save_file.0.load_globals(&save_file.1).map_err(|e| {
            shlog_error!("Failed to load globals: {}", e);
            "Failed to load globals"
        })?;

        let regions = globals.regions;
        self.output.0.clear();

        // Helper function to process a node and its children
        fn process_node(
            node_idx: usize,
            regions: &bg3_lib::lsf_reader::RegionArena,
        ) -> (String, AutoTableVar) {
            let node = regions.get_node(node_idx).unwrap();
            let mut node_table = AutoTableVar::new();

            // Add attributes
            let mut attributes = AutoTableVar::new();
            for (name, attribute) in node.attributes.iter() {
                let debug_str = format!("{:?}", attribute);
                let v = Var::ephemeral_string(debug_str.as_str());
                attributes.0.insert_fast_static(name.as_str(), &v);
            }
            let k = Var::ephemeral_string("attributes");
            node_table.0.emplace_table(k, attributes);

            // Process children recursively
            let mut children_table = AutoTableVar::new();
            for (name, child_indices) in node.children.iter() {
                let mut child_list = AutoSeqVar::new();
                for child_idx in child_indices.iter() {
                    let (child_name, child_node) = process_node(*child_idx, regions);
                    let k = Var::ephemeral_string(child_name.as_str());
                    child_list.0.emplace_table(child_node);
                }
                let k = Var::ephemeral_string(name);
                children_table.0.emplace_seq(k, child_list);
            }
            let k = Var::ephemeral_string("children");
            node_table.0.emplace_table(k, children_table);

            (node.name.clone(), node_table)
        }

        // Process all region nodes
        for (name, node_idx) in &regions.regions_indices {
            let (name, node_table) = process_node(*node_idx, &regions);
            let k = Var::ephemeral_string(name.as_str());
            self.output.0.emplace_table(k, node_table);
        }

        Ok(Some(self.output.0 .0))
    }
}

#[ctor]
fn register_shards_on_load() {
    shards::core::init();

    shlog!("shards-bg3 loaded");

    register_shard::<BG3LoadSaveFile>();
    register_shard::<BG3Globals>();
}
