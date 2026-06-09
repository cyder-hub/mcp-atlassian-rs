use super::*;

impl JiraClient {
    pub async fn get_issue_proforma_forms(
        &self,
        issue_key: String,
        cloud_id: Option<&str>,
    ) -> Result<Value, AtlassianError> {
        let Some(cloud_id) = forms_cloud_id_or_unavailable(cloud_id)? else {
            return Ok(forms_cloud_id_missing_result());
        };
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        let path = forms_cloud_api_path(&cloud_id, &format!("/issue/{issue_key}/form"));
        self.atlassian_api_http
            .send_json(self.atlassian_api_http.get(&path)?)
            .await
            .or_else(jira_forms_unavailable)
    }

    pub async fn get_proforma_form_details(
        &self,
        issue_key: String,
        form_id: String,
        cloud_id: Option<&str>,
    ) -> Result<Value, AtlassianError> {
        let Some(cloud_id) = forms_cloud_id_or_unavailable(cloud_id)? else {
            return Ok(forms_cloud_id_missing_result());
        };
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        let form_id = safe_path_segment(&form_id, "form_id")?;
        let path = forms_cloud_api_path(&cloud_id, &format!("/issue/{issue_key}/form/{form_id}"));
        self.atlassian_api_http
            .send_json(self.atlassian_api_http.get(&path)?)
            .await
            .or_else(jira_forms_unavailable)
    }

    pub async fn update_proforma_form_answers(
        &self,
        issue_key: String,
        form_id: String,
        answers: Vec<Value>,
        cloud_id: Option<&str>,
    ) -> Result<Value, AtlassianError> {
        let Some(cloud_id) = forms_cloud_id_or_unavailable(cloud_id)? else {
            return Ok(forms_cloud_id_missing_result());
        };
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        let form_id = safe_path_segment(&form_id, "form_id")?;
        let payload = proforma_answers_payload(answers)?;
        let path = forms_cloud_api_path(&cloud_id, &format!("/issue/{issue_key}/form/{form_id}"));
        self.atlassian_api_http
            .send_json_value_or_null(self.atlassian_api_http.put_json(&path, &payload)?)
            .await
            .or_else(jira_forms_unavailable)
    }
}
