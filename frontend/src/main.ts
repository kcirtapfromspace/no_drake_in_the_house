import App from './App.svelte';

const app = new App({
	target: document.body,
	props: {
		name: 'Music Streaming Blocklist Manager'
	}
});

export default app;