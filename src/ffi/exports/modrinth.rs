#![allow(clippy::missing_safety_doc)]

use super::helpers::*;
#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_search(params_json: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        use crate::extra::modrinth::dto::SearchParams;
        let raw = unsafe { cstr_to_string(params_json) }?;
        let params: SearchParams =
            serde_json::from_str(&raw).map_err(|e| format!("invalid params json: {e}"))?;
        let query = crate::extra::modrinth::build_query(&params);
        let response = block_on(crate::extra::modrinth::client::get_json::<
            crate::extra::modrinth::dto::SearchResponse,
        >("/search", &query))
        .map_err(|e| e.to_string())?;
        json_out(&response)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_project(id_or_slug: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let id = unsafe { cstr_to_string(id_or_slug) }?;
        let project = block_on(crate::extra::modrinth::project::get_project(&id))
            .map_err(|e| e.to_string())?;
        json_out(&project)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_project_by_hash(hash: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let h = unsafe { cstr_to_string(hash) }?;
        let project = block_on(crate::extra::modrinth::project::get_project_by_hash(&h))
            .map_err(|e| e.to_string())?;
        json_out(&project)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_project_versions(project_id: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let id = unsafe { cstr_to_string(project_id) }?;
        let versions = block_on(crate::extra::modrinth::version::get_project_versions(
            &id,
            &crate::extra::modrinth::dto::VersionFilters::default(),
        ))
        .map_err(|e| e.to_string())?;
        json_out(&versions)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_version(version_id: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let id = unsafe { cstr_to_string(version_id) }?;
        let v = block_on(crate::extra::modrinth::version::get_version(&id))
            .map_err(|e| e.to_string())?;
        json_out(&v)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_version_by_hash(hash: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let h = unsafe { cstr_to_string(hash) }?;
        let v = block_on(crate::extra::modrinth::version::get_version_by_hash(&h))
            .map_err(|e| e.to_string())?;
        json_out(&v)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_dependencies(project_id: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let id = unsafe { cstr_to_string(project_id) }?;
        let deps = block_on(crate::extra::modrinth::project::get_project_dependencies(
            &id,
        ))
        .map_err(|e| e.to_string())?;
        json_out(&deps)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_members(project_id: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let id = unsafe { cstr_to_string(project_id) }?;
        let m = block_on(crate::extra::modrinth::project::get_project_members(&id))
            .map_err(|e| e.to_string())?;
        json_out(&m)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_user(id_or_username: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let id = unsafe { cstr_to_string(id_or_username) }?;
        let u = block_on(crate::extra::modrinth::user::get_user(&id)).map_err(|e| e.to_string())?;
        json_out(&u)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_user_projects(user_id: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let id = unsafe { cstr_to_string(user_id) }?;
        let p = block_on(crate::extra::modrinth::user::get_user_projects(&id))
            .map_err(|e| e.to_string())?;
        json_out(&p)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_team_members(team_id: *const c_char) -> *mut c_char {
    catch_cstr(|| {
        let id = unsafe { cstr_to_string(team_id) }?;
        let m = block_on(crate::extra::modrinth::user::get_team_members(&id))
            .map_err(|e| e.to_string())?;
        json_out(&m)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_categories() -> *mut c_char {
    catch_cstr(|| {
        let c =
            block_on(crate::extra::modrinth::tag::get_categories()).map_err(|e| e.to_string())?;
        json_out(&c)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_loaders() -> *mut c_char {
    catch_cstr(|| {
        let l = block_on(crate::extra::modrinth::tag::get_loaders()).map_err(|e| e.to_string())?;
        json_out(&l)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_game_versions() -> *mut c_char {
    catch_cstr(|| {
        let g = block_on(crate::extra::modrinth::tag::get_game_versions())
            .map_err(|e| e.to_string())?;
        json_out(&g)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_install(
    id_or_slug: *const c_char,
    params_json: *const c_char,
) -> *mut c_char {
    catch_cstr(|| {
        let id = unsafe { cstr_to_string(id_or_slug) }?;
        let raw = unsafe { cstr_to_string(params_json) }?;
        let params: crate::extra::modrinth::dto::InstallParams =
            serde_json::from_str(&raw).map_err(|e| format!("invalid params json: {e}"))?;
        let r = block_on(crate::extra::modrinth::install::install_project(
            &id, &params,
        ))
        .map_err(|e| e.to_string())?;
        json_out(&r)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_install_version(
    version_id: *const c_char,
    params_json: *const c_char,
) -> *mut c_char {
    catch_cstr(|| {
        let id = unsafe { cstr_to_string(version_id) }?;
        let raw = unsafe { cstr_to_string(params_json) }?;
        let params: crate::extra::modrinth::dto::InstallParams =
            serde_json::from_str(&raw).map_err(|e| format!("invalid params json: {e}"))?;
        let r = block_on(crate::extra::modrinth::install::install_version(
            &id, &params,
        ))
        .map_err(|e| e.to_string())?;
        json_out(&r)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_install_from_hash(
    hash: *const c_char,
    params_json: *const c_char,
) -> *mut c_char {
    catch_cstr(|| {
        let h = unsafe { cstr_to_string(hash) }?;
        let raw = unsafe { cstr_to_string(params_json) }?;
        let params: crate::extra::modrinth::dto::InstallParams =
            serde_json::from_str(&raw).map_err(|e| format!("invalid params json: {e}"))?;
        let r = block_on(crate::extra::modrinth::install::install_from_hash(
            &h, &params,
        ))
        .map_err(|e| e.to_string())?;
        json_out(&r)
    })
}

#[cfg(feature = "modrinth")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rmlib_modrinth_install_datapack(
    id_or_slug: *const c_char,
    world_name: *const c_char,
    game_path: *const c_char,
    params_json: *const c_char,
) -> *mut c_char {
    catch_cstr(|| {
        let id = unsafe { cstr_to_string(id_or_slug) }?;
        let world = unsafe { cstr_to_string(world_name) }?;
        let gp = unsafe { cstr_to_string(game_path) }?;
        let raw = unsafe { cstr_to_string(params_json) }?;
        let params: crate::extra::modrinth::dto::InstallParams =
            serde_json::from_str(&raw).map_err(|e| format!("invalid params json: {e}"))?;
        let r = block_on(crate::extra::modrinth::install::install_datapack(
            &id,
            &world,
            std::path::Path::new(&gp),
            &params,
        ))
        .map_err(|e| e.to_string())?;
        json_out(&r)
    })
}
