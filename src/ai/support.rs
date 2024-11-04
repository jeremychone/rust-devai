use crate::agent::Agent;

/// Return the genai info of a particular agent.
pub fn get_genai_info(agent: &Agent) -> String {
	let mut genai_infos: Vec<String> = vec![];

	if let Some(temp) = agent.config().temperature() {
		genai_infos.push(format!("temperature: {temp}"));
	}

	if genai_infos.is_empty() {
		"".to_string()
	} else {
		format!(" ({})", genai_infos.join(", "))
	}
}
