// Utils
pub fn get_repo_name(repo: String) -> Result<String, &'static str> {
    if &repo[repo.len()-4..] == ".git" {
        let answer = &repo[..repo.len()-4];
        return Ok(answer.to_string());
    } else {
        return Err("requested repo should have .git suffix");
    }
}
