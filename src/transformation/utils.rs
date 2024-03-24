use crate::{
    infra::pi::{Sink, Source},
    SeResult,
};

pub async fn transport_source_to_sink<T>(
    source: &dyn Source<T>,
    sink: &dyn Sink<T>,
) -> SeResult<()> {
    let mut reader = source.create_reader().await?;
    let mut writer = sink.create_write_collector().await?;
    loop {
        if !reader.next(&mut writer).await? {
            break;
        }
    }
    Ok(())
}

pub fn get_type_of<T>(_: &T) -> &str {
    std::any::type_name::<T>()
}
