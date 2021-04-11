<template>
	<div id="login-root">
		<h1>{{ $t('message.login_title') }}</h1>
		<b-form @submit="onSubmit" v-if="status !== 'LoggedIn'">
			<b-alert variant="success" :show="showSuccessBadge">{{ success_msg }}</b-alert>
			<b-alert variant="danger" :show="showErrorBadge">{{ error_msg }}</b-alert>
			<b-form-group
				id="input-group-username"
				:label="$t('message.login_handle')"
				label-for="input-username">
				<b-form-input
					id="input-username"
					type="text"
					v-model="username"
					:state="stateUsername"
					:disabled="disableUsername"
					aria-describedby="input-username-help input-username-feedback"
					required></b-form-input>
				<b-form-invalid-feedback id="input-username-feedback">
					{{ usernameFeedback }}
				</b-form-invalid-feedback>
				<b-form-text id="input-username-help">
					{{ realUsernameHelp }}
				</b-form-text>
			</b-form-group>
			<b-form-group
				v-if="showPassword"
				id="input-group-password"
				:label="$t('message.password')"
				label-for="input-password">
				<b-form-input
					id="input-password"
					type="password"
					v-model="password"
					:state="statePassword"
					:disabled="disablePassword"
					aria-describedby="input-password-help input-password-feedback"
					required
					></b-form-input>
				<b-form-invalid-feedback id="input-password-feedback">
					{{ passwordFeedback }}
				</b-form-invalid-feedback>
				<b-form-text id="input-password-help">
					{{ passwordHelp }}
				</b-form-text>
			</b-form-group>
			<div align="center">
				<b-button type="submit" variant="primary" :disabled="waiting_api" v-if="!waiting_api">{{ $t('message.next') }} <b-icon icon="arrow-right"></b-icon></b-button>
				<b-spinner variant="primary" label="Loading..." v-if="waiting_api"></b-spinner>
			</div>
		</b-form>
		<p v-if="status === 'LoggedIn'">{{ $t('message.hello_name', {name: this.userDisplayName}) }}</p>
	</div>
</template>

<script>
import axios from "axios";
import globals from "@/globals.js"

// @ is an alias to /src
// import HelloWorld from '@/components/HelloWorld.vue'

export default {
	name: 'Login',
	data() {
		return {
			username: '',
			password: '',
			code2fa: '',
			success_msg: '',
			error_msg: '',
			status: 'MissingUsername',
			usernameFeedback: '',
			passwordFeedback: '',
			userDisplayName: '',
			waiting_api: false
		}
	},
	computed: {
		showSuccessBadge: function() {
			return this.success_msg.length !== 0
		},
		showErrorBadge: function() {
			return this.error_msg.length !== 0
		},
		disableUsername: function() {
			return this.waiting_api || (this.status !== 'MissingUsername' && this.status !== 'UserNotFound')
		},
		disablePassword: function() {
			return this.waiting_api
		},
		showPassword: function() {
			return this.status == 'MissingPassword' || this.status == 'WrongPassword'
		},
		stateUsername: function() {
			if (this.usernameFeedback.length !== 0) {
				return false;
			}
			return null;
		},
		statePassword: function() {
			if (this.passwordFeedback.length !== 0) {
				return false;
			}
			return null;
		},
		realUsernameHelp: function() {
			if (this.usernameHelp.length !== 0) {
				return this.usernameHelp;
			} else {
				return this.$t('message.default_login_handle_help');
			}
		}
	},
	props: {
		usernameHelp: {
			type: String,
			default: ''
		},
		passwordHelp: {
			type: String,
			default: ''
		}
	},
	methods: {
		clear() {
			this.success_msg = '';
			this.error_msg = '';
			this.usernameFeedback = '';
			this.passwordFeedback = '';
		},
		onSubmit(event) {
			event.preventDefault();
			let reqData = {
				username: this.username,
				password: this.password,
				code_otp: '',
				code_u2f: '',
				selection_2fa: '',
				remember_me: false
			};
			let endpoint = process.env.VUE_APP_API_ADDR;
			this.clear();
			
			this.waiting_api = true;
			axios.post(endpoint+"/login", reqData).then(response => {
				this.waiting_api = false;
				this.status = response.data.status;

				if (this.status === "MissingUsername") {
					// no need to handle this
				} else if (this.status === "UserNotFound") {
					this.usernameFeedback = this.$t('message.no_such_user');
					console.log(response);
				} else if (this.status === "MissingPassword") {
					// no need to handle this
				} else if (this.status === "WrongPassword") {
					this.passwordFeedback = this.$t('message.wrong_password');
					console.log(response);
				} else if (this.status === "Select2FA") {
					this.error_msg = 'Not implemented: '+this.status;
				} else if (this.status === "Wrong2FA") {
					this.error_msg = 'Not implemented: '+this.status;
				} else if (this.status === "LoggedIn") {
					this.userDisplayName = response.data.user.display_name;
				} else  {
					this.error_msg = 'Something went wrong.';
					console.log("Unexpected status: "+this.status)
					console.log(response);
				}
			}).catch(err => {
				console.log(err);
				this.waiting_api = false;
				this.error_msg = globals.axios_error_to_string(this, err);
			});
		}
	}
}
</script>

<style lang="scss" scoped>
@import '@/global-style.scss';
#login-root {
	background-color: $gray-200;
	border-radius: 0.5rem;
	width: 30rem;
	padding: 1rem;
}
</style>
