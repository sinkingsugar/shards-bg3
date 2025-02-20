use std::path::Path;

use crate::core::register_shard;
use crate::shard::Shard;
use bg3_lib::package::Package;
use bg3_lib::package_reader::PackageReader;
use ctor::*;
use lazy_static::lazy_static;
use shards::types::{
    ClonedVar, Context, ExposedTypes, InstanceData, Type, Types, Var, FRAG_CC, NONE_TYPES,
    STRING_TYPES,
};
use shards::*;

lazy_static! {
    pub static ref PACKAGE_TYPE: Type = Type::object(FRAG_CC + 1, fourCharacterCode(*b"PKG "));
}

struct LarianSaveFile(Package);
ref_counted_object_type_impl!(LarianSaveFile);

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
        &STRING_TYPES
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
        let mut package = PackageReader::new(path).map_err(|e| {
            shlog_error!("Failed to open package: {}", e);
            "Failed to open package"
        })?;
        let package = package.read().map_err(|e| {
            shlog_error!("Failed to read package: {}", e);
            "Failed to read package"
        })?;
        self.package = Var::new_ref_counted(LarianSaveFile(package), &PACKAGE_TYPE).into();
        Ok(Some(self.package.0))
    }
}

#[ctor]
fn register_shards_on_load() {
    shards::core::init();

    shlog!("shards-bg3 loaded");

    register_shard::<BG3LoadSaveFile>();
}
