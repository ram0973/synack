import { api } from '$lib/api';

let authenticated = $state(false);
let defaultPassword = $state(false);
let checked = $state(false);

export const auth = {
	get value() {
		return authenticated;
	},
	get defaultPassword() {
		return defaultPassword;
	},
	get checked() {
		return checked;
	},
	set(value: boolean) {
		authenticated = value;
		checked = true;
	},
	async refresh() {
		try {
			const result = await api.me();
			authenticated = result.authenticated;
			defaultPassword = result.default_password;
		} catch {
			authenticated = false;
			defaultPassword = false;
		} finally {
			checked = true;
		}
	}
};
