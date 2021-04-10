<template>
	<div id="login-root">
		<b-form @submit="onSubmit">
			<h1>Feroauth Login</h1>
			<b-alert variant="success" :show="showSuccessBadge">{{ success_msg }}</b-alert>
			<b-alert variant="danger" :show="showErrorBadge">{{ error_msg }}</b-alert>
			<b-form-group
				id="input-group-username"
				label="Login handle"
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
					{{ username_feedback }}
				</b-form-invalid-feedback>
				<b-form-text id="input-username-feedback">
					{{ username_desc }}
				</b-form-text>
			</b-form-group>
			<b-form-group
				v-if="showPassword"
				id="input-group-password"
				label="Password"
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
					{{ password_feedback }}
				</b-form-invalid-feedback>
				<b-form-text id="input-password-feedback">
					{{ password_desc }}
				</b-form-text>
			</b-form-group>
			<div align="center">
				<b-button type="submit" variant="primary" :disabled="waiting_api">Next</b-button>
			</div>
		</b-form>
	</div>
</template>

<script>
import axios from "axios";

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
			username_feedback: '',
			password_feedback: '',
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
			if (this.username_feedback.length !== 0) {
				return false;
			}
			return null;
		},
		statePassword: function() {
			if (this.password_feedback.length !== 0) {
				return false;
			}
			return null;
		}
	},
	props: {
		username_desc: {
			type: String,
			default: 'Email address or username'
		}
	},
	methods: {
		clear() {
			this.success_msg = '';
			this.error_msg = '';
			this.username_feedback = '';
			this.password_feedback = '';
		},
		onSubmit(event) {
			event.preventDefault();
			console.log(this);
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
					this.username_feedback = 'User not found';
					console.log(response);
				} else if (this.status === "MissingPassword") {
					// no need to handle this
				} else if (this.status === "WrongPassword") {
					this.password_feedback = 'Wrong password';
					console.log(response);
				} else if (this.status === "Select2FA") {
					this.error_msg = 'Not implemented: '+this.status;
				} else if (this.status === "Wrong2FA") {
					this.error_msg = 'Not implemented: '+this.status;
				} else if (this.status === "LoggedIn") {
					this.success_msg = 'Successfully logged in as '+response.data.user.display_name+'.';
				} else  {
					this.error_msg = 'Something went wrong.';
					console.log("Unexpected status: "+this.status)
					console.log(response);
				}
			}).catch(err => {
				this.waiting_api = false;
				this.error_msg = 'Something went wrong.';
				console.log(err);
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
