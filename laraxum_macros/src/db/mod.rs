mod stage1;
mod stage2;
mod stage3;
mod stage4;

pub fn db(
    attr: proc_macro2::TokenStream,
    input: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    // stage 1: frontend -> syntax
    let stage1_db_attr = stage1::DbAttr::try_from(attr)?;
    let stage1_db = syn::parse2::<stage1::Db>(input)?;
    // stage 2: frontend -> processing
    let stage2_db = stage2::Db::try_new(stage1_db, stage1_db_attr)?;

    // stage 3: backend -> processing
    let stage3_db = stage3::Db::try_from(&stage2_db)?;
    // stage 4: backend -> syntax
    let stage4_db = stage4::Db::from(stage3_db);

    Ok(stage4_db)
}
