let tenant = $state('Plantastic');
let tenantId = $state('00000000-0000-0000-0000-000000000001');
let authToken = $state('');
let activeProjectId = $state<string | null>(null);

export const session = {
	get tenant() {
		return tenant;
	},
	set tenant(v: string) {
		tenant = v;
	},
	get tenantId() {
		return tenantId;
	},
	set tenantId(v: string) {
		tenantId = v;
	},
	get authToken() {
		return authToken;
	},
	set authToken(v: string) {
		authToken = v;
	},
	get activeProjectId() {
		return activeProjectId;
	},
	set activeProjectId(v: string | null) {
		activeProjectId = v;
	}
};
