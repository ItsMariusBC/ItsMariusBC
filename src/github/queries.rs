//! GraphQL query strings. Port of inline queries + `queries.ts`.

pub const STARS_REPOS_QUERY: &str = r#"
	query ($owner_affiliation: [RepositoryAffiliation], $login: String!, $cursor: String) {
		user(login: $login) {
			repositories(first: 100, after: $cursor, ownerAffiliations: $owner_affiliation) {
				totalCount
				edges {
					node {
						... on Repository {
							nameWithOwner
							stargazers {
								totalCount
							}
						}
					}
				}
				pageInfo {
					endCursor
					hasNextPage
				}
			}
		}
	}"#;

pub const FOLLOWERS_QUERY: &str = r#"
	query($login: String!){
		user(login: $login) {
			followers {
				totalCount
			}
		}
	}"#;

pub const LOC_QUERY: &str = r#"
	query ($owner_affiliation: [RepositoryAffiliation], $login: String!, $cursor: String) {
		user(login: $login) {
			repositories(first: 50, after: $cursor, ownerAffiliations: $owner_affiliation) {
				edges {
					node {
						... on Repository {
							nameWithOwner
							defaultBranchRef {
								target {
									... on Commit {
										history {
											totalCount
										}
									}
								}
							}
						}
					}
				}
				pageInfo {
					endCursor
					hasNextPage
				}
			}
		}
	}"#;

pub const RECURSIVE_LOC_QUERY: &str = r#"
	query ($repo_name: String!, $owner: String!, $cursor: String) {
		repository(name: $repo_name, owner: $owner) {
			defaultBranchRef {
				target {
					... on Commit {
						history(first: 100, after: $cursor) {
							totalCount
							edges {
								node {
									... on Commit {
										committedDate
									}
									author {
										user {
											id
										}
									}
									deletions
									additions
								}
							}
							pageInfo {
								endCursor
								hasNextPage
							}
						}
					}
				}
			}
		}
	}
"#;
