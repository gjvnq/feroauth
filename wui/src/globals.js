const axios_error_to_string = (vue, err) => {
	if (!err.status) {
		return vue.$t('message.cant_talk_net_down');
	} else if (err.status == 404) {
		return vue.$t('message.err_http_404');
	} else if (err.status == 403) {
		return vue.$t('message.err_http_403');
	} else if (err.status == 500) {
		return vue.$t('message.err_http_500');
	} else {
		return vue.$t('message.err_http_other', {code: err.status});
	}
};

exports.axios_error_to_string = axios_error_to_string;