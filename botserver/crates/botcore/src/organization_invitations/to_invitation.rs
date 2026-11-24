use super::*;

impl InvitationRow {
    pub(super) fn to_invitation(&self) -> OrganizationInvitation {
        OrganizationInvitation {
            id: self.id,
            organization_id: self.org_id,
            email: self.email.clone(),
            role: self.role.parse().unwrap_or(InvitationRole::Member),
            groups: match self.groups.as_array() {
                Some(groups) => groups
                    .iter()
                    .filter_map(|value| value.as_str().map(String::from))
                    .collect(),
                None => Vec::new(),
            },
            invited_by: self.invited_by,
            invited_by_name: match &self.invited_by_name {
                Some(name) => name.clone(),
                None => String::new(),
            },
            status: match self.status.as_str() {
                "accepted" => InvitationStatus::Accepted,
                "declined" => InvitationStatus::Declined,
                "expired" => InvitationStatus::Expired,
                "revoked" => InvitationStatus::Revoked,
                _ => InvitationStatus::Pending,
            },
            token: match &self.token {
                Some(token) => token.clone(),
                None => String::new(),
            },
            message: self.message.clone(),
            expires_at: self.expires_at.unwrap_or_else(Utc::now),
            created_at: self.created_at,
            updated_at: self.updated_at.unwrap_or(self.created_at),
            accepted_at: self.accepted_at,
            accepted_by: self.accepted_by,
        }
    }
}
