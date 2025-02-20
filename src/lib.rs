use crate::core::register_shard;
use crate::shard::Shard;
use ctor::*;
use shards::types::{Context, ExposedTypes, InstanceData, Type, Types, Var, NONE_TYPES};
use shards::*;

#[derive(shards::shard)]
#[shard_info("BG3", "AddDescriptionHere")]
struct BG3Shard {
    #[shard_required]
    required: ExposedTypes,
}

impl Default for BG3Shard {
    fn default() -> Self {
        Self {
            required: ExposedTypes::new(),
        }
    }
}

#[shards::shard_impl]
impl Shard for BG3Shard {
    fn input_types(&mut self) -> &Types {
        &NONE_TYPES
    }

    fn output_types(&mut self) -> &Types {
        &NONE_TYPES
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

    fn activate(&mut self, _context: &Context, _input: &Var) -> Result<Option<SHVar>, &str> {
        Ok(Some(Var::default()))
    }
}

#[ctor]
fn register_shards_on_load() {
    shards::core::init();

    println!("shards-bg3 loaded");

    register_shard::<BG3Shard>();
}
