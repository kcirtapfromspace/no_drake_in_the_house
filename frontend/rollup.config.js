import svelte from 'rollup-plugin-svelte';
import commonjs from '@rollup/plugin-commonjs';
import resolve from '@rollup/plugin-node-resolve';
import livereload from 'rollup-plugin-livereload';
import terser from '@rollup/plugin-terser';
import replace from '@rollup/plugin-replace';
import sveltePreprocess from 'svelte-preprocess';
import typescript from '@rollup/plugin-typescript';
import css from 'rollup-plugin-css-only';
import { spawn } from 'child_process';
import tailwindcss from 'tailwindcss';
import autoprefixer from 'autoprefixer';
import { readFileSync } from 'fs';
import { resolve as pathResolve } from 'path';

// Load environment variables from .env file
function loadEnv() {
	try {
		const envPath = pathResolve('.env');
		const envFile = readFileSync(envPath, 'utf8');
		const envVars = {};
		
		envFile.split('\n').forEach(line => {
			const [key, ...valueParts] = line.split('=');
			if (key && valueParts.length > 0) {
				const value = valueParts.join('=').trim();
				envVars[key.trim()] = value;
			}
		});
		
		return envVars;
	} catch (error) {
		console.warn('Could not load .env file:', error.message);
		return {};
	}
}

const envVars = loadEnv();

const production = !process.env.ROLLUP_WATCH;

function serve() {
	let server;

	function toExit() {
		if (server) server.kill(0);
	}

	return {
		writeBundle() {
			if (server) return;
			server = spawn('npm', ['run', 'start', '--', '--dev'], {
				stdio: ['ignore', 'inherit', 'inherit'],
				shell: true
			});

			process.on('SIGTERM', toExit);
			process.on('exit', toExit);
		}
	};
}

export default {
	input: 'src/main.ts',
	output: {
		sourcemap: true,
		format: 'iife',
		name: 'app',
		file: 'public/build/bundle.js'
	},
	plugins: [
		replace({
			preventAssignment: true,
			values: {
				'import.meta.env.VITE_API_URL': JSON.stringify(process.env.VITE_API_URL || envVars.VITE_API_URL || 'http://localhost:3000'),
				'import.meta.env.VITE_API_VERSION': JSON.stringify(process.env.VITE_API_VERSION || envVars.VITE_API_VERSION || 'v1'),
				'import.meta.env.VITE_APP_NAME': JSON.stringify(process.env.VITE_APP_NAME || envVars.VITE_APP_NAME || 'No Drake in the House'),
				'import.meta.env.VITE_ENVIRONMENT': JSON.stringify(process.env.VITE_ENVIRONMENT || envVars.VITE_ENVIRONMENT || 'development'),
				'import.meta.env.VITE_ENABLE_2FA': JSON.stringify(process.env.VITE_ENABLE_2FA || envVars.VITE_ENABLE_2FA || 'true'),
				'import.meta.env.VITE_ENABLE_COMMUNITY_LISTS': JSON.stringify(process.env.VITE_ENABLE_COMMUNITY_LISTS || envVars.VITE_ENABLE_COMMUNITY_LISTS || 'true'),
				'import.meta.env.VITE_ENABLE_ANALYTICS': JSON.stringify(process.env.VITE_ENABLE_ANALYTICS || envVars.VITE_ENABLE_ANALYTICS || 'false'),
				'import.meta.env.VITE_HOT_RELOAD': JSON.stringify(process.env.VITE_HOT_RELOAD || envVars.VITE_HOT_RELOAD || 'true'),
				'import.meta.env.VITE_DEBUG_MODE': JSON.stringify(process.env.VITE_DEBUG_MODE || envVars.VITE_DEBUG_MODE || 'true'),
				'import.meta.env.VITE_SPOTIFY_CLIENT_ID': JSON.stringify(process.env.VITE_SPOTIFY_CLIENT_ID || envVars.VITE_SPOTIFY_CLIENT_ID || ''),
				'import.meta.env.VITE_APPLE_MUSIC_DEVELOPER_TOKEN': JSON.stringify(process.env.VITE_APPLE_MUSIC_DEVELOPER_TOKEN || envVars.VITE_APPLE_MUSIC_DEVELOPER_TOKEN || ''),
				'import.meta.env.VITE_DEFAULT_THEME': JSON.stringify(process.env.VITE_DEFAULT_THEME || envVars.VITE_DEFAULT_THEME || 'light'),
				'import.meta.env.VITE_ENABLE_SERVICE_WORKER': JSON.stringify(process.env.VITE_ENABLE_SERVICE_WORKER || envVars.VITE_ENABLE_SERVICE_WORKER || 'false'),
				'import.meta.env.VITE_CACHE_DURATION': JSON.stringify(process.env.VITE_CACHE_DURATION || envVars.VITE_CACHE_DURATION || '300000'),
			}
		}),
		svelte({
			preprocess: sveltePreprocess({
				sourceMap: !production,
				postcss: {
					plugins: [tailwindcss, autoprefixer],
				},
			}),
			compilerOptions: {
				dev: !production
			}
		}),
		css({ output: 'bundle.css' }),

		resolve({
			browser: true,
			dedupe: ['svelte'],
			exportConditions: ['svelte']
		}),
		commonjs(),
		typescript({
			sourceMap: !production,
			inlineSources: !production
		}),

		!production && serve(),
		!production && livereload('public'),
		production && terser()
	],
	watch: {
		clearScreen: false
	}
};