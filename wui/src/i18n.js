import Vue from 'vue'
import VueI18n from 'vue-i18n'

Vue.use(VueI18n)

const messages = {
	en: {
		message: {
			login_title: 'Feroauth Login',
			login_handle: 'Login Handle',
			next: 'Next',
			password: 'Password',
			hello_name: 'Hello {name}!',
			no_such_user: 'No such user.',
			wrong_password: 'Wrong password.',
			generic_error: 'Something went wrong.',
			cant_talk_net_down: 'Can\'t talk to the server. Are you connected to the Internet?',
			err_http_404: 'Resource not found.',
			err_http_403: 'You are not allowed to do that.',
			err_http_500: 'Something went wrong inside the server.',
			err_http_other: 'Something went wrong. HTTP error code: {code}.',
			default_login_handle_help: 'Email or username or user UUID'
		}
	},
	pt: {
		message: {
			login_title: 'Login Feroauth',
			login_handle: 'Alça de Login',
			next: 'Próximo',
			password: 'Senha',
			no_such_user: 'Tal usuário inexiste.',
			wrong_password: 'Senha incorreta.',
			hello_name: 'Olá {name}!',
			generic_error: 'Algo deu errado.',
			cant_talk_net_down: 'Falha ao falar com o servidor. Você está conectado à Internet?',
			err_http_404: 'Recurso não encontrado.',
			err_http_403: 'Você não tem permissão para isso.',
			err_http_500: 'Algo deu errado dentro no servidor.',
			err_http_other: 'Algo deu errado. Código de erro HTTP: {code}.',
			default_login_handle_help: 'Email ou nome de usuário ou UUID do usuário'
		}
	}
};

export default new VueI18n({
	locale: process.env.VUE_APP_I18N_LOCALE || 'en',
	fallbackLocale: process.env.VUE_APP_I18N_FALLBACK_LOCALE || 'en',
	messages: messages
})
