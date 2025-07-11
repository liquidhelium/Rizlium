use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use zip::write::FileOptions;

pub fn pack_files_into_zip<P: AsRef<Path>>(
    files: HashMap<String, Vec<u8>>,
    output_path: P,
) -> zip::result::ZipResult<()> {
    let file = File::create(output_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options: FileOptions<'_, ()> =
        FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for (name, data) in files {
        zip.start_file(name, options)?;
        zip.write_all(&data)?;
    }
    zip.finish()?;
    Ok(())
}
