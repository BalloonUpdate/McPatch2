use crate::data::index_file::IndexFile;
use crate::AppContext;

pub fn do_combine(ctx: AppContext, new_label: &str) -> i32 {
    let mut index_file = IndexFile::load(&ctx.index_file_internal);

    if index_file.contains_label(new_label) {
        println!("");
        return 1;
    }

    todo!();

    0
}