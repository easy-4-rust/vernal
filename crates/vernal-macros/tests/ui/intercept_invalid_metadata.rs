struct InvalidMetadataService;

impl InvalidMetadataService {
    #[vernal_macros::intercept(tags = ["needs review"])]
    async fn execute(
        self: std::sync::Arc<Self>,
    ) -> Result<(), vernal_aop::InvocationError> {
        Ok(())
    }
}

fn main() {}
