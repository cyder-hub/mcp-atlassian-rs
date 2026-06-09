use super::*;

impl ConfluenceClient {
    pub async fn search_user(
        &self,
        cql: &str,
        limit: Option<u64>,
        group_name: Option<&str>,
    ) -> Result<ConfluenceUserSearchResponse, AtlassianError> {
        let cql = required_non_empty_input(cql, "query")?;
        let limit = user_search_limit(limit)?;

        match self.config.deployment {
            ConfluenceDeployment::Cloud => {
                self.get_json(
                    "/rest/api/search/user",
                    vec![
                        ("cql".to_string(), cql),
                        ("limit".to_string(), limit.to_string()),
                    ],
                )
                .await
            }
            ConfluenceDeployment::ServerDataCenter => {
                self.search_user_server_dc(&cql, group_name, limit).await
            }
        }
    }

    async fn search_user_server_dc(
        &self,
        cql: &str,
        group_name: Option<&str>,
        limit: u64,
    ) -> Result<ConfluenceUserSearchResponse, AtlassianError> {
        let group_name = required_non_empty_input(
            group_name.unwrap_or(DEFAULT_CONFLUENCE_GROUP_NAME),
            "group_name",
        )?;
        let search_term = extract_user_fullname_search_term(cql).unwrap_or(cql);
        let search_lower = search_term.to_ascii_lowercase();
        let mut start = 0;
        let mut matches = Vec::new();

        while matches.len() < limit as usize {
            let encoded_group = percent_encode_path_segment(&group_name);
            let response: ConfluenceUserListResponse = self
                .get_json(
                    &format!("/rest/api/group/{encoded_group}/member"),
                    vec![
                        ("start".to_string(), start.to_string()),
                        (
                            "limit".to_string(),
                            SERVER_USER_SEARCH_PAGE_SIZE.to_string(),
                        ),
                    ],
                )
                .await?;
            let member_count = response.results.len() as u64;

            for user in response.results {
                let display_name = user.display_name.clone().unwrap_or_default();
                let username = user.username.clone().unwrap_or_default();
                if display_name.to_ascii_lowercase().contains(&search_lower)
                    || username.to_ascii_lowercase().contains(&search_lower)
                {
                    matches.push(ConfluenceUserSearchResult::from_user(
                        user,
                        Some(display_name),
                    ));
                    if matches.len() >= limit as usize {
                        break;
                    }
                }
            }

            if member_count == 0 || response.links.get("next").is_none() {
                break;
            }
            start += member_count;
        }

        Ok(ConfluenceUserSearchResponse {
            start: Some(0),
            limit: Some(limit),
            size: Some(matches.len() as u64),
            total_size: Some(matches.len() as u64),
            cql_query: Some(cql.to_string()),
            results: matches,
            ..ConfluenceUserSearchResponse::default()
        })
    }
}
