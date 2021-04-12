import axios from 'axios';

const state = () => ({
	data: {}
});

// getters
const getters = {};

// actions
const actions = {
	loadFromServer ({ commit }) {
		axios.get('/api/session/info', {withCredentials: true}).then(response => {
			commit('setSessionData', response.data);
		});
	}
};

// mutations
const mutations = {
	setSessionData(state, data) {
		console.log('setSessionData', data);
		state.data = data
	},
};

export default {
	namespaced: true,
	state,
	getters,
	actions,
	mutations
};
