
(function(l, r) { if (!l || l.getElementById('livereloadscript')) return; r = l.createElement('script'); r.async = 1; r.src = '//' + (self.location.host || 'localhost').split(':')[0] + ':35734/livereload.js?snipver=1'; r.id = 'livereloadscript'; l.getElementsByTagName('head')[0].appendChild(r) })(self.document);
var app = (function () {
	'use strict';

	/** @returns {void} */
	function noop() {}

	/** @returns {void} */
	function add_location(element, file, line, column, char) {
		element.__svelte_meta = {
			loc: { file, line, column, char }
		};
	}

	function run(fn) {
		return fn();
	}

	function blank_object() {
		return Object.create(null);
	}

	/**
	 * @param {Function[]} fns
	 * @returns {void}
	 */
	function run_all(fns) {
		fns.forEach(run);
	}

	/**
	 * @param {any} thing
	 * @returns {thing is Function}
	 */
	function is_function(thing) {
		return typeof thing === 'function';
	}

	/** @returns {boolean} */
	function safe_not_equal(a, b) {
		return a != a ? b == b : a !== b || (a && typeof a === 'object') || typeof a === 'function';
	}

	let src_url_equal_anchor;

	/**
	 * @param {string} element_src
	 * @param {string} url
	 * @returns {boolean}
	 */
	function src_url_equal(element_src, url) {
		if (element_src === url) return true;
		if (!src_url_equal_anchor) {
			src_url_equal_anchor = document.createElement('a');
		}
		// This is actually faster than doing URL(..).href
		src_url_equal_anchor.href = url;
		return element_src === src_url_equal_anchor.href;
	}

	/** @returns {boolean} */
	function is_empty(obj) {
		return Object.keys(obj).length === 0;
	}

	/** @returns {void} */
	function validate_store(store, name) {
		if (store != null && typeof store.subscribe !== 'function') {
			throw new Error(`'${name}' is not a store with a 'subscribe' method`);
		}
	}

	function subscribe(store, ...callbacks) {
		if (store == null) {
			for (const callback of callbacks) {
				callback(undefined);
			}
			return noop;
		}
		const unsub = store.subscribe(...callbacks);
		return unsub.unsubscribe ? () => unsub.unsubscribe() : unsub;
	}

	/** @returns {void} */
	function component_subscribe(component, store, callback) {
		component.$$.on_destroy.push(subscribe(store, callback));
	}

	/** @type {typeof globalThis} */
	const globals =
		typeof window !== 'undefined'
			? window
			: typeof globalThis !== 'undefined'
			? globalThis
			: // @ts-ignore Node typings have this
			  global;

	/**
	 * @param {Node} target
	 * @param {Node} node
	 * @returns {void}
	 */
	function append(target, node) {
		target.appendChild(node);
	}

	/**
	 * @param {Node} target
	 * @param {Node} node
	 * @param {Node} [anchor]
	 * @returns {void}
	 */
	function insert(target, node, anchor) {
		target.insertBefore(node, anchor || null);
	}

	/**
	 * @param {Node} node
	 * @returns {void}
	 */
	function detach(node) {
		if (node.parentNode) {
			node.parentNode.removeChild(node);
		}
	}

	/**
	 * @returns {void} */
	function destroy_each(iterations, detaching) {
		for (let i = 0; i < iterations.length; i += 1) {
			if (iterations[i]) iterations[i].d(detaching);
		}
	}

	/**
	 * @template {keyof HTMLElementTagNameMap} K
	 * @param {K} name
	 * @returns {HTMLElementTagNameMap[K]}
	 */
	function element(name) {
		return document.createElement(name);
	}

	/**
	 * @template {keyof SVGElementTagNameMap} K
	 * @param {K} name
	 * @returns {SVGElement}
	 */
	function svg_element(name) {
		return document.createElementNS('http://www.w3.org/2000/svg', name);
	}

	/**
	 * @param {string} data
	 * @returns {Text}
	 */
	function text(data) {
		return document.createTextNode(data);
	}

	/**
	 * @returns {Text} */
	function space() {
		return text(' ');
	}

	/**
	 * @returns {Text} */
	function empty() {
		return text('');
	}

	/**
	 * @param {EventTarget} node
	 * @param {string} event
	 * @param {EventListenerOrEventListenerObject} handler
	 * @param {boolean | AddEventListenerOptions | EventListenerOptions} [options]
	 * @returns {() => void}
	 */
	function listen(node, event, handler, options) {
		node.addEventListener(event, handler, options);
		return () => node.removeEventListener(event, handler, options);
	}

	/**
	 * @returns {(event: any) => any} */
	function prevent_default(fn) {
		return function (event) {
			event.preventDefault();
			// @ts-ignore
			return fn.call(this, event);
		};
	}

	/**
	 * @param {Element} node
	 * @param {string} attribute
	 * @param {string} [value]
	 * @returns {void}
	 */
	function attr(node, attribute, value) {
		if (value == null) node.removeAttribute(attribute);
		else if (node.getAttribute(attribute) !== value) node.setAttribute(attribute, value);
	}

	/**
	 * @param {Element} element
	 * @returns {ChildNode[]}
	 */
	function children(element) {
		return Array.from(element.childNodes);
	}

	/**
	 * @returns {void} */
	function set_input_value(input, value) {
		input.value = value == null ? '' : value;
	}

	/**
	 * @template T
	 * @param {string} type
	 * @param {T} [detail]
	 * @param {{ bubbles?: boolean, cancelable?: boolean }} [options]
	 * @returns {CustomEvent<T>}
	 */
	function custom_event(type, detail, { bubbles = false, cancelable = false } = {}) {
		return new CustomEvent(type, { detail, bubbles, cancelable });
	}

	/**
	 * @typedef {Node & {
	 * 	claim_order?: number;
	 * 	hydrate_init?: true;
	 * 	actual_end_child?: NodeEx;
	 * 	childNodes: NodeListOf<NodeEx>;
	 * }} NodeEx
	 */

	/** @typedef {ChildNode & NodeEx} ChildNodeEx */

	/** @typedef {NodeEx & { claim_order: number }} NodeEx2 */

	/**
	 * @typedef {ChildNodeEx[] & {
	 * 	claim_info?: {
	 * 		last_index: number;
	 * 		total_claimed: number;
	 * 	};
	 * }} ChildNodeArray
	 */

	let current_component;

	/** @returns {void} */
	function set_current_component(component) {
		current_component = component;
	}

	function get_current_component() {
		if (!current_component) throw new Error('Function called outside component initialization');
		return current_component;
	}

	/**
	 * The `onMount` function schedules a callback to run as soon as the component has been mounted to the DOM.
	 * It must be called during the component's initialisation (but doesn't need to live *inside* the component;
	 * it can be called from an external module).
	 *
	 * If a function is returned _synchronously_ from `onMount`, it will be called when the component is unmounted.
	 *
	 * `onMount` does not run inside a [server-side component](https://svelte.dev/docs#run-time-server-side-component-api).
	 *
	 * https://svelte.dev/docs/svelte#onmount
	 * @template T
	 * @param {() => import('./private.js').NotFunction<T> | Promise<import('./private.js').NotFunction<T>> | (() => any)} fn
	 * @returns {void}
	 */
	function onMount(fn) {
		get_current_component().$$.on_mount.push(fn);
	}

	const dirty_components = [];
	const binding_callbacks = [];

	let render_callbacks = [];

	const flush_callbacks = [];

	const resolved_promise = /* @__PURE__ */ Promise.resolve();

	let update_scheduled = false;

	/** @returns {void} */
	function schedule_update() {
		if (!update_scheduled) {
			update_scheduled = true;
			resolved_promise.then(flush);
		}
	}

	/** @returns {void} */
	function add_render_callback(fn) {
		render_callbacks.push(fn);
	}

	// flush() calls callbacks in this order:
	// 1. All beforeUpdate callbacks, in order: parents before children
	// 2. All bind:this callbacks, in reverse order: children before parents.
	// 3. All afterUpdate callbacks, in order: parents before children. EXCEPT
	//    for afterUpdates called during the initial onMount, which are called in
	//    reverse order: children before parents.
	// Since callbacks might update component values, which could trigger another
	// call to flush(), the following steps guard against this:
	// 1. During beforeUpdate, any updated components will be added to the
	//    dirty_components array and will cause a reentrant call to flush(). Because
	//    the flush index is kept outside the function, the reentrant call will pick
	//    up where the earlier call left off and go through all dirty components. The
	//    current_component value is saved and restored so that the reentrant call will
	//    not interfere with the "parent" flush() call.
	// 2. bind:this callbacks cannot trigger new flush() calls.
	// 3. During afterUpdate, any updated components will NOT have their afterUpdate
	//    callback called a second time; the seen_callbacks set, outside the flush()
	//    function, guarantees this behavior.
	const seen_callbacks = new Set();

	let flushidx = 0; // Do *not* move this inside the flush() function

	/** @returns {void} */
	function flush() {
		// Do not reenter flush while dirty components are updated, as this can
		// result in an infinite loop. Instead, let the inner flush handle it.
		// Reentrancy is ok afterwards for bindings etc.
		if (flushidx !== 0) {
			return;
		}
		const saved_component = current_component;
		do {
			// first, call beforeUpdate functions
			// and update components
			try {
				while (flushidx < dirty_components.length) {
					const component = dirty_components[flushidx];
					flushidx++;
					set_current_component(component);
					update(component.$$);
				}
			} catch (e) {
				// reset dirty state to not end up in a deadlocked state and then rethrow
				dirty_components.length = 0;
				flushidx = 0;
				throw e;
			}
			set_current_component(null);
			dirty_components.length = 0;
			flushidx = 0;
			while (binding_callbacks.length) binding_callbacks.pop()();
			// then, once components are updated, call
			// afterUpdate functions. This may cause
			// subsequent updates...
			for (let i = 0; i < render_callbacks.length; i += 1) {
				const callback = render_callbacks[i];
				if (!seen_callbacks.has(callback)) {
					// ...so guard against infinite loops
					seen_callbacks.add(callback);
					callback();
				}
			}
			render_callbacks.length = 0;
		} while (dirty_components.length);
		while (flush_callbacks.length) {
			flush_callbacks.pop()();
		}
		update_scheduled = false;
		seen_callbacks.clear();
		set_current_component(saved_component);
	}

	/** @returns {void} */
	function update($$) {
		if ($$.fragment !== null) {
			$$.update();
			run_all($$.before_update);
			const dirty = $$.dirty;
			$$.dirty = [-1];
			$$.fragment && $$.fragment.p($$.ctx, dirty);
			$$.after_update.forEach(add_render_callback);
		}
	}

	/**
	 * Useful for example to execute remaining `afterUpdate` callbacks before executing `destroy`.
	 * @param {Function[]} fns
	 * @returns {void}
	 */
	function flush_render_callbacks(fns) {
		const filtered = [];
		const targets = [];
		render_callbacks.forEach((c) => (fns.indexOf(c) === -1 ? filtered.push(c) : targets.push(c)));
		targets.forEach((c) => c());
		render_callbacks = filtered;
	}

	const outroing = new Set();

	/**
	 * @type {Outro}
	 */
	let outros;

	/**
	 * @returns {void} */
	function group_outros() {
		outros = {
			r: 0,
			c: [],
			p: outros // parent group
		};
	}

	/**
	 * @returns {void} */
	function check_outros() {
		if (!outros.r) {
			run_all(outros.c);
		}
		outros = outros.p;
	}

	/**
	 * @param {import('./private.js').Fragment} block
	 * @param {0 | 1} [local]
	 * @returns {void}
	 */
	function transition_in(block, local) {
		if (block && block.i) {
			outroing.delete(block);
			block.i(local);
		}
	}

	/**
	 * @param {import('./private.js').Fragment} block
	 * @param {0 | 1} local
	 * @param {0 | 1} [detach]
	 * @param {() => void} [callback]
	 * @returns {void}
	 */
	function transition_out(block, local, detach, callback) {
		if (block && block.o) {
			if (outroing.has(block)) return;
			outroing.add(block);
			outros.c.push(() => {
				outroing.delete(block);
				if (callback) {
					if (detach) block.d(1);
					callback();
				}
			});
			block.o(local);
		} else if (callback) {
			callback();
		}
	}

	/** @typedef {1} INTRO */
	/** @typedef {0} OUTRO */
	/** @typedef {{ direction: 'in' | 'out' | 'both' }} TransitionOptions */
	/** @typedef {(node: Element, params: any, options: TransitionOptions) => import('../transition/public.js').TransitionConfig} TransitionFn */

	/**
	 * @typedef {Object} Outro
	 * @property {number} r
	 * @property {Function[]} c
	 * @property {Object} p
	 */

	/**
	 * @typedef {Object} PendingProgram
	 * @property {number} start
	 * @property {INTRO|OUTRO} b
	 * @property {Outro} [group]
	 */

	/**
	 * @typedef {Object} Program
	 * @property {number} a
	 * @property {INTRO|OUTRO} b
	 * @property {1|-1} d
	 * @property {number} duration
	 * @property {number} start
	 * @property {number} end
	 * @property {Outro} [group]
	 */

	// general each functions:

	function ensure_array_like(array_like_or_iterator) {
		return array_like_or_iterator?.length !== undefined
			? array_like_or_iterator
			: Array.from(array_like_or_iterator);
	}

	/** @returns {void} */
	function create_component(block) {
		block && block.c();
	}

	/** @returns {void} */
	function mount_component(component, target, anchor) {
		const { fragment, after_update } = component.$$;
		fragment && fragment.m(target, anchor);
		// onMount happens before the initial afterUpdate
		add_render_callback(() => {
			const new_on_destroy = component.$$.on_mount.map(run).filter(is_function);
			// if the component was destroyed immediately
			// it will update the `$$.on_destroy` reference to `null`.
			// the destructured on_destroy may still reference to the old array
			if (component.$$.on_destroy) {
				component.$$.on_destroy.push(...new_on_destroy);
			} else {
				// Edge case - component was destroyed immediately,
				// most likely as a result of a binding initialising
				run_all(new_on_destroy);
			}
			component.$$.on_mount = [];
		});
		after_update.forEach(add_render_callback);
	}

	/** @returns {void} */
	function destroy_component(component, detaching) {
		const $$ = component.$$;
		if ($$.fragment !== null) {
			flush_render_callbacks($$.after_update);
			run_all($$.on_destroy);
			$$.fragment && $$.fragment.d(detaching);
			// TODO null out other refs, including component.$$ (but need to
			// preserve final state?)
			$$.on_destroy = $$.fragment = null;
			$$.ctx = [];
		}
	}

	/** @returns {void} */
	function make_dirty(component, i) {
		if (component.$$.dirty[0] === -1) {
			dirty_components.push(component);
			schedule_update();
			component.$$.dirty.fill(0);
		}
		component.$$.dirty[(i / 31) | 0] |= 1 << i % 31;
	}

	// TODO: Document the other params
	/**
	 * @param {SvelteComponent} component
	 * @param {import('./public.js').ComponentConstructorOptions} options
	 *
	 * @param {import('./utils.js')['not_equal']} not_equal Used to compare props and state values.
	 * @param {(target: Element | ShadowRoot) => void} [append_styles] Function that appends styles to the DOM when the component is first initialised.
	 * This will be the `add_css` function from the compiled component.
	 *
	 * @returns {void}
	 */
	function init(
		component,
		options,
		instance,
		create_fragment,
		not_equal,
		props,
		append_styles = null,
		dirty = [-1]
	) {
		const parent_component = current_component;
		set_current_component(component);
		/** @type {import('./private.js').T$$} */
		const $$ = (component.$$ = {
			fragment: null,
			ctx: [],
			// state
			props,
			update: noop,
			not_equal,
			bound: blank_object(),
			// lifecycle
			on_mount: [],
			on_destroy: [],
			on_disconnect: [],
			before_update: [],
			after_update: [],
			context: new Map(options.context || (parent_component ? parent_component.$$.context : [])),
			// everything else
			callbacks: blank_object(),
			dirty,
			skip_bound: false,
			root: options.target || parent_component.$$.root
		});
		append_styles && append_styles($$.root);
		let ready = false;
		$$.ctx = instance
			? instance(component, options.props || {}, (i, ret, ...rest) => {
					const value = rest.length ? rest[0] : ret;
					if ($$.ctx && not_equal($$.ctx[i], ($$.ctx[i] = value))) {
						if (!$$.skip_bound && $$.bound[i]) $$.bound[i](value);
						if (ready) make_dirty(component, i);
					}
					return ret;
			  })
			: [];
		$$.update();
		ready = true;
		run_all($$.before_update);
		// `false` as a special case of no DOM component
		$$.fragment = create_fragment ? create_fragment($$.ctx) : false;
		if (options.target) {
			if (options.hydrate) {
				// TODO: what is the correct type here?
				// @ts-expect-error
				const nodes = children(options.target);
				$$.fragment && $$.fragment.l(nodes);
				nodes.forEach(detach);
			} else {
				// eslint-disable-next-line @typescript-eslint/no-non-null-assertion
				$$.fragment && $$.fragment.c();
			}
			if (options.intro) transition_in(component.$$.fragment);
			mount_component(component, options.target, options.anchor);
			flush();
		}
		set_current_component(parent_component);
	}

	/**
	 * Base class for Svelte components. Used when dev=false.
	 *
	 * @template {Record<string, any>} [Props=any]
	 * @template {Record<string, any>} [Events=any]
	 */
	class SvelteComponent {
		/**
		 * ### PRIVATE API
		 *
		 * Do not use, may change at any time
		 *
		 * @type {any}
		 */
		$$ = undefined;
		/**
		 * ### PRIVATE API
		 *
		 * Do not use, may change at any time
		 *
		 * @type {any}
		 */
		$$set = undefined;

		/** @returns {void} */
		$destroy() {
			destroy_component(this, 1);
			this.$destroy = noop;
		}

		/**
		 * @template {Extract<keyof Events, string>} K
		 * @param {K} type
		 * @param {((e: Events[K]) => void) | null | undefined} callback
		 * @returns {() => void}
		 */
		$on(type, callback) {
			if (!is_function(callback)) {
				return noop;
			}
			const callbacks = this.$$.callbacks[type] || (this.$$.callbacks[type] = []);
			callbacks.push(callback);
			return () => {
				const index = callbacks.indexOf(callback);
				if (index !== -1) callbacks.splice(index, 1);
			};
		}

		/**
		 * @param {Partial<Props>} props
		 * @returns {void}
		 */
		$set(props) {
			if (this.$$set && !is_empty(props)) {
				this.$$.skip_bound = true;
				this.$$set(props);
				this.$$.skip_bound = false;
			}
		}
	}

	/**
	 * @typedef {Object} CustomElementPropDefinition
	 * @property {string} [attribute]
	 * @property {boolean} [reflect]
	 * @property {'String'|'Boolean'|'Number'|'Array'|'Object'} [type]
	 */

	// generated during release, do not modify

	/**
	 * The current version, as set in package.json.
	 *
	 * https://svelte.dev/docs/svelte-compiler#svelte-version
	 * @type {string}
	 */
	const VERSION = '4.2.20';
	const PUBLIC_VERSION = '4';

	/**
	 * @template T
	 * @param {string} type
	 * @param {T} [detail]
	 * @returns {void}
	 */
	function dispatch_dev(type, detail) {
		document.dispatchEvent(custom_event(type, { version: VERSION, ...detail }, { bubbles: true }));
	}

	/**
	 * @param {Node} target
	 * @param {Node} node
	 * @returns {void}
	 */
	function append_dev(target, node) {
		dispatch_dev('SvelteDOMInsert', { target, node });
		append(target, node);
	}

	/**
	 * @param {Node} target
	 * @param {Node} node
	 * @param {Node} [anchor]
	 * @returns {void}
	 */
	function insert_dev(target, node, anchor) {
		dispatch_dev('SvelteDOMInsert', { target, node, anchor });
		insert(target, node, anchor);
	}

	/**
	 * @param {Node} node
	 * @returns {void}
	 */
	function detach_dev(node) {
		dispatch_dev('SvelteDOMRemove', { node });
		detach(node);
	}

	/**
	 * @param {Node} node
	 * @param {string} event
	 * @param {EventListenerOrEventListenerObject} handler
	 * @param {boolean | AddEventListenerOptions | EventListenerOptions} [options]
	 * @param {boolean} [has_prevent_default]
	 * @param {boolean} [has_stop_propagation]
	 * @param {boolean} [has_stop_immediate_propagation]
	 * @returns {() => void}
	 */
	function listen_dev(
		node,
		event,
		handler,
		options,
		has_prevent_default,
		has_stop_propagation,
		has_stop_immediate_propagation
	) {
		const modifiers =
			options === true ? ['capture'] : options ? Array.from(Object.keys(options)) : [];
		if (has_prevent_default) modifiers.push('preventDefault');
		if (has_stop_propagation) modifiers.push('stopPropagation');
		if (has_stop_immediate_propagation) modifiers.push('stopImmediatePropagation');
		dispatch_dev('SvelteDOMAddEventListener', { node, event, handler, modifiers });
		const dispose = listen(node, event, handler, options);
		return () => {
			dispatch_dev('SvelteDOMRemoveEventListener', { node, event, handler, modifiers });
			dispose();
		};
	}

	/**
	 * @param {Element} node
	 * @param {string} attribute
	 * @param {string} [value]
	 * @returns {void}
	 */
	function attr_dev(node, attribute, value) {
		attr(node, attribute, value);
		if (value == null) dispatch_dev('SvelteDOMRemoveAttribute', { node, attribute });
		else dispatch_dev('SvelteDOMSetAttribute', { node, attribute, value });
	}

	/**
	 * @param {Element} node
	 * @param {string} property
	 * @param {any} [value]
	 * @returns {void}
	 */
	function prop_dev(node, property, value) {
		node[property] = value;
		dispatch_dev('SvelteDOMSetProperty', { node, property, value });
	}

	/**
	 * @param {Text} text
	 * @param {unknown} data
	 * @returns {void}
	 */
	function set_data_dev(text, data) {
		data = '' + data;
		if (text.data === data) return;
		dispatch_dev('SvelteDOMSetData', { node: text, data });
		text.data = /** @type {string} */ (data);
	}

	function ensure_array_like_dev(arg) {
		if (
			typeof arg !== 'string' &&
			!(arg && typeof arg === 'object' && 'length' in arg) &&
			!(typeof Symbol === 'function' && arg && Symbol.iterator in arg)
		) {
			throw new Error('{#each} only works with iterable values.');
		}
		return ensure_array_like(arg);
	}

	/**
	 * @returns {void} */
	function validate_slots(name, slot, keys) {
		for (const slot_key of Object.keys(slot)) {
			if (!~keys.indexOf(slot_key)) {
				console.warn(`<${name}> received an unexpected slot "${slot_key}".`);
			}
		}
	}

	/**
	 * Base class for Svelte components with some minor dev-enhancements. Used when dev=true.
	 *
	 * Can be used to create strongly typed Svelte components.
	 *
	 * #### Example:
	 *
	 * You have component library on npm called `component-library`, from which
	 * you export a component called `MyComponent`. For Svelte+TypeScript users,
	 * you want to provide typings. Therefore you create a `index.d.ts`:
	 * ```ts
	 * import { SvelteComponent } from "svelte";
	 * export class MyComponent extends SvelteComponent<{foo: string}> {}
	 * ```
	 * Typing this makes it possible for IDEs like VS Code with the Svelte extension
	 * to provide intellisense and to use the component like this in a Svelte file
	 * with TypeScript:
	 * ```svelte
	 * <script lang="ts">
	 * 	import { MyComponent } from "component-library";
	 * </script>
	 * <MyComponent foo={'bar'} />
	 * ```
	 * @template {Record<string, any>} [Props=any]
	 * @template {Record<string, any>} [Events=any]
	 * @template {Record<string, any>} [Slots=any]
	 * @extends {SvelteComponent<Props, Events>}
	 */
	class SvelteComponentDev extends SvelteComponent {
		/**
		 * For type checking capabilities only.
		 * Does not exist at runtime.
		 * ### DO NOT USE!
		 *
		 * @type {Props}
		 */
		$$prop_def;
		/**
		 * For type checking capabilities only.
		 * Does not exist at runtime.
		 * ### DO NOT USE!
		 *
		 * @type {Events}
		 */
		$$events_def;
		/**
		 * For type checking capabilities only.
		 * Does not exist at runtime.
		 * ### DO NOT USE!
		 *
		 * @type {Slots}
		 */
		$$slot_def;

		/** @param {import('./public.js').ComponentConstructorOptions<Props>} options */
		constructor(options) {
			if (!options || (!options.target && !options.$$inline)) {
				throw new Error("'target' is a required option");
			}
			super();
		}

		/** @returns {void} */
		$destroy() {
			super.$destroy();
			this.$destroy = () => {
				console.warn('Component was already destroyed'); // eslint-disable-line no-console
			};
		}

		/** @returns {void} */
		$capture_state() {}

		/** @returns {void} */
		$inject_state() {}
	}

	if (typeof window !== 'undefined')
		// @ts-ignore
		(window.__svelte || (window.__svelte = { v: new Set() })).v.add(PUBLIC_VERSION);

	const subscriber_queue = [];

	/**
	 * Creates a `Readable` store that allows reading by subscription.
	 *
	 * https://svelte.dev/docs/svelte-store#readable
	 * @template T
	 * @param {T} [value] initial value
	 * @param {import('./public.js').StartStopNotifier<T>} [start]
	 * @returns {import('./public.js').Readable<T>}
	 */
	function readable(value, start) {
		return {
			subscribe: writable(value, start).subscribe
		};
	}

	/**
	 * Create a `Writable` store that allows both updating and reading by subscription.
	 *
	 * https://svelte.dev/docs/svelte-store#writable
	 * @template T
	 * @param {T} [value] initial value
	 * @param {import('./public.js').StartStopNotifier<T>} [start]
	 * @returns {import('./public.js').Writable<T>}
	 */
	function writable(value, start = noop) {
		/** @type {import('./public.js').Unsubscriber} */
		let stop;
		/** @type {Set<import('./private.js').SubscribeInvalidateTuple<T>>} */
		const subscribers = new Set();
		/** @param {T} new_value
		 * @returns {void}
		 */
		function set(new_value) {
			if (safe_not_equal(value, new_value)) {
				value = new_value;
				if (stop) {
					// store is ready
					const run_queue = !subscriber_queue.length;
					for (const subscriber of subscribers) {
						subscriber[1]();
						subscriber_queue.push(subscriber, value);
					}
					if (run_queue) {
						for (let i = 0; i < subscriber_queue.length; i += 2) {
							subscriber_queue[i][0](subscriber_queue[i + 1]);
						}
						subscriber_queue.length = 0;
					}
				}
			}
		}

		/**
		 * @param {import('./public.js').Updater<T>} fn
		 * @returns {void}
		 */
		function update(fn) {
			set(fn(value));
		}

		/**
		 * @param {import('./public.js').Subscriber<T>} run
		 * @param {import('./private.js').Invalidator<T>} [invalidate]
		 * @returns {import('./public.js').Unsubscriber}
		 */
		function subscribe(run, invalidate = noop) {
			/** @type {import('./private.js').SubscribeInvalidateTuple<T>} */
			const subscriber = [run, invalidate];
			subscribers.add(subscriber);
			if (subscribers.size === 1) {
				stop = start(set, update) || noop;
			}
			run(value);
			return () => {
				subscribers.delete(subscriber);
				if (subscribers.size === 0 && stop) {
					stop();
					stop = null;
				}
			};
		}
		return { set, update, subscribe };
	}

	/**
	 * Derived value store by synchronizing one or more readable stores and
	 * applying an aggregation function over its input values.
	 *
	 * https://svelte.dev/docs/svelte-store#derived
	 * @template {import('./private.js').Stores} S
	 * @template T
	 * @overload
	 * @param {S} stores - input stores
	 * @param {(values: import('./private.js').StoresValues<S>, set: (value: T) => void, update: (fn: import('./public.js').Updater<T>) => void) => import('./public.js').Unsubscriber | void} fn - function callback that aggregates the values
	 * @param {T} [initial_value] - initial value
	 * @returns {import('./public.js').Readable<T>}
	 */

	/**
	 * Derived value store by synchronizing one or more readable stores and
	 * applying an aggregation function over its input values.
	 *
	 * https://svelte.dev/docs/svelte-store#derived
	 * @template {import('./private.js').Stores} S
	 * @template T
	 * @overload
	 * @param {S} stores - input stores
	 * @param {(values: import('./private.js').StoresValues<S>) => T} fn - function callback that aggregates the values
	 * @param {T} [initial_value] - initial value
	 * @returns {import('./public.js').Readable<T>}
	 */

	/**
	 * @template {import('./private.js').Stores} S
	 * @template T
	 * @param {S} stores
	 * @param {Function} fn
	 * @param {T} [initial_value]
	 * @returns {import('./public.js').Readable<T>}
	 */
	function derived(stores, fn, initial_value) {
		const single = !Array.isArray(stores);
		/** @type {Array<import('./public.js').Readable<any>>} */
		const stores_array = single ? [stores] : stores;
		if (!stores_array.every(Boolean)) {
			throw new Error('derived() expects stores as input, got a falsy value');
		}
		const auto = fn.length < 2;
		return readable(initial_value, (set, update) => {
			let started = false;
			const values = [];
			let pending = 0;
			let cleanup = noop;
			const sync = () => {
				if (pending) {
					return;
				}
				cleanup();
				const result = fn(single ? values[0] : values, set, update);
				if (auto) {
					set(result);
				} else {
					cleanup = is_function(result) ? result : noop;
				}
			};
			const unsubscribers = stores_array.map((store, i) =>
				subscribe(
					store,
					(value) => {
						values[i] = value;
						pending &= ~(1 << i);
						if (started) {
							sync();
						}
					},
					() => {
						pending |= 1 << i;
					}
				)
			);
			started = true;
			sync();
			return function stop() {
				run_all(unsubscribers);
				cleanup();
				// We need to set this to false because callbacks can still happen despite having unsubscribed:
				// Callbacks might already be placed in the queue which doesn't know it should no longer
				// invoke this derived store.
				started = false;
			};
		});
	}

	/**
	 * Centralized API client for consistent request/response handling
	 * Handles authentication, error handling, and retry logic
	 */
	class ApiClient {
	    constructor(baseUrl = 'http://localhost:3000') {
	        Object.defineProperty(this, "baseUrl", {
	            enumerable: true,
	            configurable: true,
	            writable: true,
	            value: void 0
	        });
	        Object.defineProperty(this, "maxRetries", {
	            enumerable: true,
	            configurable: true,
	            writable: true,
	            value: 3
	        });
	        Object.defineProperty(this, "retryDelay", {
	            enumerable: true,
	            configurable: true,
	            writable: true,
	            value: 1000
	        }); // Base delay in ms
	        this.baseUrl = baseUrl;
	    }
	    /**
	     * Get authentication token from localStorage
	     */
	    getAuthToken() {
	        return localStorage.getItem('auth_token');
	    }
	    /**
	     * Set authentication token in localStorage
	     */
	    setAuthToken(token) {
	        localStorage.setItem('auth_token', token);
	    }
	    /**
	     * Clear authentication token
	     */
	    clearAuthToken() {
	        localStorage.removeItem('auth_token');
	    }
	    /**
	     * Create request headers with authentication
	     */
	    createHeaders(includeAuth = true) {
	        const headers = {
	            'Content-Type': 'application/json',
	        };
	        if (includeAuth) {
	            const token = this.getAuthToken();
	            if (token) {
	                headers['Authorization'] = `Bearer ${token}`;
	            }
	        }
	        return headers;
	    }
	    /**
	     * Handle API response and extract data
	     */
	    async handleResponse(response) {
	        const contentType = response.headers.get('content-type');
	        let data;
	        if (contentType && contentType.includes('application/json')) {
	            data = await response.json();
	        }
	        else {
	            data = { message: await response.text() };
	        }
	        // If response has the expected format, return it
	        if (typeof data === 'object' && data !== null && 'success' in data) {
	            return data;
	        }
	        // Handle non-standard responses
	        if (response.ok) {
	            return {
	                success: true,
	                data: data,
	                timestamp: new Date().toISOString(),
	            };
	        }
	        else {
	            return {
	                success: false,
	                message: data.message || `HTTP ${response.status}: ${response.statusText}`,
	                error_code: `HTTP_${response.status}`,
	                timestamp: new Date().toISOString(),
	            };
	        }
	    }
	    /**
	     * Retry logic with exponential backoff
	     */
	    async withRetry(operation, retries = this.maxRetries) {
	        let lastError = null;
	        for (let attempt = 0; attempt <= retries; attempt++) {
	            try {
	                const response = await operation();
	                // Don't retry on authentication errors or client errors (4xx)
	                if (response.status === 401 || response.status === 403 ||
	                    (response.status >= 400 && response.status < 500)) {
	                    return this.handleResponse(response);
	                }
	                // Don't retry on success
	                if (response.ok) {
	                    return this.handleResponse(response);
	                }
	                // Retry on server errors (5xx) or network issues
	                if (attempt < retries) {
	                    const delay = this.retryDelay * Math.pow(2, attempt);
	                    await new Promise(resolve => setTimeout(resolve, delay));
	                    continue;
	                }
	                return this.handleResponse(response);
	            }
	            catch (error) {
	                lastError = error;
	                if (attempt < retries) {
	                    const delay = this.retryDelay * Math.pow(2, attempt);
	                    await new Promise(resolve => setTimeout(resolve, delay));
	                    continue;
	                }
	            }
	        }
	        // If all retries failed, return error response
	        return {
	            success: false,
	            message: lastError?.message || 'Network request failed after retries',
	            error_code: 'NETWORK_ERROR',
	            timestamp: new Date().toISOString(),
	        };
	    }
	    /**
	     * GET request
	     */
	    async get(endpoint, includeAuth = true) {
	        const url = `${this.baseUrl}${endpoint}`;
	        return this.withRetry(() => fetch(url, {
	            method: 'GET',
	            headers: this.createHeaders(includeAuth),
	        }));
	    }
	    /**
	     * POST request
	     */
	    async post(endpoint, data, includeAuth = true) {
	        const url = `${this.baseUrl}${endpoint}`;
	        return this.withRetry(() => fetch(url, {
	            method: 'POST',
	            headers: this.createHeaders(includeAuth),
	            body: data ? JSON.stringify(data) : undefined,
	        }));
	    }
	    /**
	     * PUT request
	     */
	    async put(endpoint, data, includeAuth = true) {
	        const url = `${this.baseUrl}${endpoint}`;
	        return this.withRetry(() => fetch(url, {
	            method: 'PUT',
	            headers: this.createHeaders(includeAuth),
	            body: data ? JSON.stringify(data) : undefined,
	        }));
	    }
	    /**
	     * DELETE request
	     */
	    async delete(endpoint, includeAuth = true) {
	        const url = `${this.baseUrl}${endpoint}`;
	        return this.withRetry(() => fetch(url, {
	            method: 'DELETE',
	            headers: this.createHeaders(includeAuth),
	        }));
	    }
	    /**
	     * Handle authentication errors by attempting token refresh
	     */
	    async handleAuthError() {
	        try {
	            const refreshToken = localStorage.getItem('refresh_token');
	            if (!refreshToken) {
	                return false;
	            }
	            const response = await this.post('/api/v1/auth/refresh', { refresh_token: refreshToken }, false // Don't include auth for refresh
	            );
	            if (response.success && response.data) {
	                this.setAuthToken(response.data.access_token);
	                localStorage.setItem('refresh_token', response.data.refresh_token);
	                return true;
	            }
	            return false;
	        }
	        catch (error) {
	            console.error('Token refresh failed:', error);
	            return false;
	        }
	    }
	    /**
	     * Make authenticated request with automatic token refresh
	     */
	    async authenticatedRequest(method, endpoint, data) {
	        let response;
	        // Make initial request
	        switch (method) {
	            case 'GET':
	                response = await this.get(endpoint);
	                break;
	            case 'POST':
	                response = await this.post(endpoint, data);
	                break;
	            case 'PUT':
	                response = await this.put(endpoint, data);
	                break;
	            case 'DELETE':
	                response = await this.delete(endpoint);
	                break;
	        }
	        // If unauthorized, try to refresh token and retry once
	        if (!response.success && response.error_code === 'HTTP_401') {
	            const refreshed = await this.handleAuthError();
	            if (refreshed) {
	                // Retry the original request
	                switch (method) {
	                    case 'GET':
	                        response = await this.get(endpoint);
	                        break;
	                    case 'POST':
	                        response = await this.post(endpoint, data);
	                        break;
	                    case 'PUT':
	                        response = await this.put(endpoint, data);
	                        break;
	                    case 'DELETE':
	                        response = await this.delete(endpoint);
	                        break;
	                }
	            }
	            else {
	                // Clear tokens and redirect to login
	                this.clearAuthToken();
	                localStorage.removeItem('refresh_token');
	                // Dispatch custom event for auth failure
	                window.dispatchEvent(new CustomEvent('auth:logout', {
	                    detail: { reason: 'token_refresh_failed' }
	                }));
	            }
	        }
	        return response;
	    }
	}
	// Create singleton instance
	const apiClient = new ApiClient();

	const initialState = {
	    user: null,
	    token: localStorage.getItem('auth_token'),
	    refreshToken: localStorage.getItem('refresh_token'),
	    isAuthenticated: false,
	    isLoading: false,
	    justRegistered: false,
	    oauthFlow: {
	        provider: null,
	        state: null,
	        isInProgress: false,
	    },
	};
	const authStore = writable(initialState);
	const isAuthenticated = derived(authStore, ($auth) => $auth.isAuthenticated && $auth.token !== null);
	const currentUser = derived(authStore, ($auth) => $auth.user);
	derived(authStore, ($auth) => $auth.justRegistered);
	// Auth actions
	const authActions = {
	    login: async (email, password, totpCode) => {
	        authStore.update(state => ({ ...state, isLoading: true }));
	        const result = await apiClient.post('/api/v1/auth/login', {
	            email,
	            password,
	            totp_code: totpCode
	        }, false // Don't include auth for login
	        );
	        if (result.success && result.data) {
	            const { access_token, refresh_token } = result.data;
	            apiClient.setAuthToken(access_token);
	            localStorage.setItem('refresh_token', refresh_token);
	            authStore.update(state => ({
	                ...state,
	                token: access_token,
	                refreshToken: refresh_token,
	                isAuthenticated: true,
	                isLoading: false,
	                justRegistered: false, // Reset on login
	            }));
	            // Fetch user profile
	            await authActions.fetchProfile();
	            return { success: true };
	        }
	        else {
	            authStore.update(state => ({ ...state, isLoading: false }));
	            return { success: false, message: result.message || 'Login failed' };
	        }
	    },
	    register: async (email, password, confirmPassword, termsAccepted) => {
	        authStore.update(state => ({ ...state, isLoading: true }));
	        const result = await apiClient.post('/api/v1/auth/register', {
	            email,
	            password,
	            confirm_password: confirmPassword,
	            terms_accepted: termsAccepted
	        }, false // Don't include auth for registration
	        );
	        if (result.success) {
	            // Check if auto-login was successful (tokens returned)
	            if (result.data?.access_token && result.data?.refresh_token) {
	                const { access_token, refresh_token } = result.data;
	                apiClient.setAuthToken(access_token);
	                localStorage.setItem('refresh_token', refresh_token);
	                authStore.update(state => ({
	                    ...state,
	                    token: access_token,
	                    refreshToken: refresh_token,
	                    isAuthenticated: true,
	                    isLoading: false,
	                    justRegistered: true, // Mark as just registered for better UX
	                }));
	                // Fetch user profile
	                await authActions.fetchProfile();
	                return { success: true, autoLogin: true };
	            }
	            else {
	                authStore.update(state => ({ ...state, isLoading: false }));
	                return { success: true, autoLogin: false, message: result.message };
	            }
	        }
	        else {
	            authStore.update(state => ({ ...state, isLoading: false }));
	            return {
	                success: false,
	                message: result.message || 'Registration failed',
	                errors: result.data?.errors || null
	            };
	        }
	    },
	    fetchProfile: async () => {
	        const token = apiClient.getAuthToken();
	        if (!token)
	            return;
	        const result = await apiClient.authenticatedRequest('GET', '/api/v1/users/profile');
	        if (result.success && result.data) {
	            authStore.update(state => ({
	                ...state,
	                user: result.data,
	                isAuthenticated: true,
	            }));
	        }
	        else {
	            console.error('Failed to fetch profile:', result.message);
	        }
	    },
	    logout: async () => {
	        const token = apiClient.getAuthToken();
	        if (token) {
	            try {
	                await apiClient.authenticatedRequest('POST', '/api/v1/auth/logout');
	            }
	            catch (error) {
	                console.error('Logout request failed:', error);
	            }
	        }
	        apiClient.clearAuthToken();
	        localStorage.removeItem('refresh_token');
	        authStore.set({
	            user: null,
	            token: null,
	            refreshToken: null,
	            isAuthenticated: false,
	            isLoading: false,
	            justRegistered: false,
	            oauthFlow: {
	                provider: null,
	                state: null,
	                isInProgress: false,
	            },
	        });
	    },
	    refreshToken: async () => {
	        const refreshToken = localStorage.getItem('refresh_token');
	        if (!refreshToken)
	            return false;
	        const result = await apiClient.post('/api/v1/auth/refresh', { refresh_token: refreshToken }, false // Don't include auth for refresh
	        );
	        if (result.success && result.data) {
	            const { access_token, refresh_token: newRefreshToken } = result.data;
	            apiClient.setAuthToken(access_token);
	            localStorage.setItem('refresh_token', newRefreshToken);
	            authStore.update(state => ({
	                ...state,
	                token: access_token,
	                refreshToken: newRefreshToken,
	                isAuthenticated: true,
	            }));
	            return true;
	        }
	        else {
	            console.error('Token refresh failed:', result.message);
	            return false;
	        }
	    },
	    // 2FA Management
	    setup2FA: async () => {
	        try {
	            const result = await api.post('/auth/2fa/setup');
	            if (result.success) {
	                return {
	                    success: true,
	                    qrCodeUrl: result.data.qr_code_url,
	                    secret: result.data.secret
	                };
	            }
	            else {
	                return { success: false, message: result.message };
	            }
	        }
	        catch (error) {
	            return { success: false, message: error.message || 'Failed to setup 2FA' };
	        }
	    },
	    verify2FA: async (code) => {
	        try {
	            const result = await api.post('/auth/2fa/verify', { totp_code: code });
	            if (result.success) {
	                // Update user profile to reflect 2FA is now enabled
	                await authActions.fetchProfile();
	                return { success: true };
	            }
	            else {
	                return { success: false, message: result.message };
	            }
	        }
	        catch (error) {
	            return { success: false, message: error.message || 'Failed to verify 2FA code' };
	        }
	    },
	    disable2FA: async (code) => {
	        try {
	            const result = await api.post('/auth/2fa/disable', { totp_code: code });
	            if (result.success) {
	                // Update user profile to reflect 2FA is now disabled
	                await authActions.fetchProfile();
	                return { success: true };
	            }
	            else {
	                return { success: false, message: result.message };
	            }
	        }
	        catch (error) {
	            return { success: false, message: error.message || 'Failed to disable 2FA' };
	        }
	    },
	    // Clear the just registered flag (useful for onboarding flows)
	    clearJustRegistered: () => {
	        authStore.update(state => ({
	            ...state,
	            justRegistered: false,
	        }));
	    },
	    // OAuth-specific actions
	    initiateOAuthFlow: async (provider) => {
	        authStore.update(state => ({
	            ...state,
	            oauthFlow: {
	                provider,
	                state: null,
	                isInProgress: true,
	            },
	        }));
	        try {
	            const result = await api.post(`/auth/oauth/${provider}/initiate`);
	            if (result.success) {
	                const { authorization_url, state } = result.data;
	                // Store state for validation
	                sessionStorage.setItem(`oauth_state_${provider}`, state);
	                authStore.update(authState => ({
	                    ...authState,
	                    oauthFlow: {
	                        ...authState.oauthFlow,
	                        state,
	                    },
	                }));
	                // Redirect to OAuth provider
	                window.location.href = authorization_url;
	                return { success: true };
	            }
	            else {
	                authStore.update(state => ({
	                    ...state,
	                    oauthFlow: {
	                        provider: null,
	                        state: null,
	                        isInProgress: false,
	                    },
	                }));
	                return { success: false, message: result.message };
	            }
	        }
	        catch (error) {
	            authStore.update(state => ({
	                ...state,
	                oauthFlow: {
	                    provider: null,
	                    state: null,
	                    isInProgress: false,
	                },
	            }));
	            return { success: false, message: error.message || 'Network error occurred' };
	        }
	    },
	    completeOAuthFlow: async (provider, code, state) => {
	        try {
	            // Validate state parameter
	            const storedState = sessionStorage.getItem(`oauth_state_${provider}`);
	            if (!storedState || storedState !== state) {
	                throw new Error('Invalid state parameter - possible CSRF attack');
	            }
	            const result = await api.post(`/auth/oauth/${provider}/callback`, {
	                code,
	                state,
	                redirect_uri: window.location.origin + window.location.pathname,
	            });
	            if (result.success) {
	                const { access_token, refresh_token } = result.data;
	                // Store tokens
	                localStorage.setItem('auth_token', access_token);
	                localStorage.setItem('refresh_token', refresh_token);
	                // Update auth store
	                authStore.update(authState => ({
	                    ...authState,
	                    token: access_token,
	                    refreshToken: refresh_token,
	                    isAuthenticated: true,
	                    oauthFlow: {
	                        provider: null,
	                        state: null,
	                        isInProgress: false,
	                    },
	                }));
	                // Clean up stored state
	                sessionStorage.removeItem(`oauth_state_${provider}`);
	                // Fetch user profile
	                await authActions.fetchProfile();
	                return { success: true };
	            }
	            else {
	                throw new Error(result.message || 'OAuth authentication failed');
	            }
	        }
	        catch (error) {
	            // Clean up on error
	            sessionStorage.removeItem(`oauth_state_${provider}`);
	            authStore.update(state => ({
	                ...state,
	                oauthFlow: {
	                    provider: null,
	                    state: null,
	                    isInProgress: false,
	                },
	            }));
	            return { success: false, message: error.message || 'Authentication failed' };
	        }
	    },
	    linkOAuthAccount: async (provider) => {
	        try {
	            const result = await api.post(`/auth/oauth/${provider}/link`);
	            if (result.success) {
	                const { authorization_url, state } = result.data;
	                // Store state for validation
	                sessionStorage.setItem(`oauth_link_state_${provider}`, state);
	                return {
	                    success: true,
	                    authorization_url,
	                    state
	                };
	            }
	            else {
	                return { success: false, message: result.message };
	            }
	        }
	        catch (error) {
	            return { success: false, message: error.message || 'Network error occurred' };
	        }
	    },
	    unlinkOAuthAccount: async (provider) => {
	        try {
	            const result = await api.delete(`/auth/oauth/${provider}/unlink`);
	            if (result.success) {
	                // Refresh user profile to update linked accounts
	                await authActions.fetchProfile();
	                return { success: true };
	            }
	            else {
	                return { success: false, message: result.message };
	            }
	        }
	        catch (error) {
	            return { success: false, message: error.message || 'Network error occurred' };
	        }
	    },
	    getLinkedAccounts: async () => {
	        try {
	            const result = await api.get('/auth/oauth/accounts');
	            if (result.success) {
	                return { success: true, data: result.data };
	            }
	            else {
	                return { success: false, message: result.message };
	            }
	        }
	        catch (error) {
	            return { success: false, message: error.message || 'Network error occurred' };
	        }
	    },
	};
	// Initialize auth state on app load
	if (typeof window !== 'undefined') {
	    const token = localStorage.getItem('auth_token');
	    if (token) {
	        apiClient.setAuthToken(token);
	        authStore.update(state => ({
	            ...state,
	            token,
	            isAuthenticated: true,
	            justRegistered: false, // Reset on app load
	        }));
	        authActions.fetchProfile();
	    }
	    // Listen for auth logout events from API client
	    window.addEventListener('auth:logout', () => {
	        authActions.logout();
	    });
	}

	// Path mappings
	const pathToRoute = {
	    '/': 'home',
	    '/home': 'home',
	    '/settings': 'settings',
	};
	const routeToPath = {
	    'home': '/',
	    'settings': '/settings',
	    'oauth-callback': '/auth/callback',
	    'oauth-error': '/auth/error',
	};
	// Route metadata
	const routeMeta = {
	    'home': { title: 'Home', description: 'Your music blocklist dashboard' },
	    'settings': { title: 'Settings', description: 'Account and connection settings' },
	    'oauth-callback': { title: 'Connecting...', description: 'Processing authentication' },
	    'oauth-error': { title: 'Connection Error', description: 'There was a problem connecting' },
	};
	// Router store
	const currentRoute = writable('home');
	// Derived store for route metadata
	derived(currentRoute, $route => routeMeta[$route]);
	// Navigation function
	function navigateTo(route) {
	    currentRoute.set(route);
	    if (typeof window !== 'undefined') {
	        const path = routeToPath[route] || '/';
	        const meta = routeMeta[route];
	        window.history.pushState({ route }, meta.title, path);
	        document.title = `${meta.title} - No Drake`;
	    }
	}
	// Get route from path
	function getRouteFromPath(path) {
	    if (path.startsWith('/auth/callback'))
	        return 'oauth-callback';
	    if (path.startsWith('/auth/error'))
	        return 'oauth-error';
	    return pathToRoute[path] || 'home';
	}
	// Initialize router
	function initRouter() {
	    if (typeof window !== 'undefined') {
	        window.addEventListener('popstate', (event) => {
	            const route = event.state?.route || getRouteFromPath(window.location.pathname);
	            currentRoute.set(route);
	        });
	        const initialRoute = getRouteFromPath(window.location.pathname);
	        currentRoute.set(initialRoute);
	        document.title = `${routeMeta[initialRoute].title} - No Drake`;
	    }
	}

	/* src/lib/components/Login.svelte generated by Svelte v4.2.20 */
	const file$4 = "src/lib/components/Login.svelte";

	// (63:6) {#if error}
	function create_if_block_3$3(ctx) {
		let div;
		let t;

		const block = {
			c: function create() {
				div = element("div");
				t = text(/*error*/ ctx[2]);
				attr_dev(div, "class", "bg-red-900/50 border border-red-700 text-red-200 px-4 py-3 rounded-lg text-sm");
				add_location(div, file$4, 65, 8, 1806);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, t);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*error*/ 4) set_data_dev(t, /*error*/ ctx[2]);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_3$3.name,
			type: "if",
			source: "(63:6) {#if error}",
			ctx
		});

		return block;
	}

	// (69:6) {#if success}
	function create_if_block_2$3(ctx) {
		let div;
		let t;

		const block = {
			c: function create() {
				div = element("div");
				t = text(/*success*/ ctx[3]);
				attr_dev(div, "class", "bg-green-900/50 border border-green-700 text-green-200 px-4 py-3 rounded-lg text-sm");
				add_location(div, file$4, 71, 8, 1972);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, t);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*success*/ 8) set_data_dev(t, /*success*/ ctx[3]);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_2$3.name,
			type: "if",
			source: "(69:6) {#if success}",
			ctx
		});

		return block;
	}

	// (96:6) {#if mode === 'register'}
	function create_if_block_1$4(ctx) {
		let div;
		let input;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				div = element("div");
				input = element("input");
				attr_dev(input, "type", "password");
				attr_dev(input, "placeholder", "Confirm password");
				input.required = true;
				attr_dev(input, "minlength", "8");
				attr_dev(input, "class", "w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-red-500 focus:border-transparent");
				add_location(input, file$4, 99, 10, 2891);
				add_location(div, file$4, 98, 8, 2875);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, input);
				set_input_value(input, /*confirmPassword*/ ctx[6]);

				if (!mounted) {
					dispose = listen_dev(input, "input", /*input_input_handler*/ ctx[11]);
					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (dirty & /*confirmPassword*/ 64 && input.value !== /*confirmPassword*/ ctx[6]) {
					set_input_value(input, /*confirmPassword*/ ctx[6]);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}

				mounted = false;
				dispose();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_1$4.name,
			type: "if",
			source: "(96:6) {#if mode === 'register'}",
			ctx
		});

		return block;
	}

	// (114:8) {#if isLoading}
	function create_if_block$4(ctx) {
		let div;

		const block = {
			c: function create() {
				div = element("div");
				attr_dev(div, "class", "w-5 h-5 border-2 border-white border-t-transparent rounded-full animate-spin mr-2");
				add_location(div, file$4, 116, 10, 3585);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block$4.name,
			type: "if",
			source: "(114:8) {#if isLoading}",
			ctx
		});

		return block;
	}

	function create_fragment$4(ctx) {
		let div9;
		let div8;
		let div0;
		let span0;
		let t1;
		let h1;
		let t3;
		let p;
		let t5;
		let form;
		let t6;
		let t7;
		let div1;
		let input0;
		let t8;
		let div2;
		let input1;
		let t9;
		let t10;
		let button0;
		let t11;

		let t12_value = (/*mode*/ ctx[0] === 'login'
		? 'Sign in'
		: 'Create account') + "";

		let t12;
		let t13;
		let div3;
		let button1;

		let t14_value = (/*mode*/ ctx[0] === 'login'
		? "Don't have an account? Sign up"
		: 'Already have an account? Sign in') + "";

		let t14;
		let t15;
		let div7;
		let div4;
		let svg0;
		let path0;
		let t16;
		let span1;
		let t18;
		let div5;
		let svg1;
		let path1;
		let t19;
		let span2;
		let t21;
		let div6;
		let svg2;
		let path2;
		let t22;
		let span3;
		let mounted;
		let dispose;
		let if_block0 = /*error*/ ctx[2] && create_if_block_3$3(ctx);
		let if_block1 = /*success*/ ctx[3] && create_if_block_2$3(ctx);
		let if_block2 = /*mode*/ ctx[0] === 'register' && create_if_block_1$4(ctx);
		let if_block3 = /*isLoading*/ ctx[1] && create_if_block$4(ctx);

		const block = {
			c: function create() {
				div9 = element("div");
				div8 = element("div");
				div0 = element("div");
				span0 = element("span");
				span0.textContent = "🚫";
				t1 = space();
				h1 = element("h1");
				h1.textContent = "No Drake";
				t3 = space();
				p = element("p");
				p.textContent = "Block problematic artists from your music";
				t5 = space();
				form = element("form");
				if (if_block0) if_block0.c();
				t6 = space();
				if (if_block1) if_block1.c();
				t7 = space();
				div1 = element("div");
				input0 = element("input");
				t8 = space();
				div2 = element("div");
				input1 = element("input");
				t9 = space();
				if (if_block2) if_block2.c();
				t10 = space();
				button0 = element("button");
				if (if_block3) if_block3.c();
				t11 = space();
				t12 = text(t12_value);
				t13 = space();
				div3 = element("div");
				button1 = element("button");
				t14 = text(t14_value);
				t15 = space();
				div7 = element("div");
				div4 = element("div");
				svg0 = svg_element("svg");
				path0 = svg_element("path");
				t16 = space();
				span1 = element("span");
				span1.textContent = "AI-curated blocklists by category";
				t18 = space();
				div5 = element("div");
				svg1 = svg_element("svg");
				path1 = svg_element("path");
				t19 = space();
				span2 = element("span");
				span2.textContent = "Block artists on Spotify & Apple Music";
				t21 = space();
				div6 = element("div");
				svg2 = svg_element("svg");
				path2 = svg_element("path");
				t22 = space();
				span3 = element("span");
				span3.textContent = "Blocks features & collaborations too";
				attr_dev(span0, "class", "text-6xl");
				add_location(span0, file$4, 57, 6, 1499);
				attr_dev(h1, "class", "text-3xl font-bold text-white mt-4");
				add_location(h1, file$4, 58, 6, 1538);
				attr_dev(p, "class", "text-gray-400 mt-2");
				add_location(p, file$4, 59, 6, 1605);
				attr_dev(div0, "class", "text-center mb-8");
				add_location(div0, file$4, 56, 4, 1462);
				attr_dev(input0, "type", "email");
				attr_dev(input0, "placeholder", "Email");
				input0.required = true;
				attr_dev(input0, "class", "w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-red-500 focus:border-transparent");
				add_location(input0, file$4, 77, 8, 2138);
				add_location(div1, file$4, 76, 6, 2124);
				attr_dev(input1, "type", "password");
				attr_dev(input1, "placeholder", "Password");
				input1.required = true;
				attr_dev(input1, "minlength", "8");
				attr_dev(input1, "class", "w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-red-500 focus:border-transparent");
				add_location(input1, file$4, 87, 8, 2480);
				add_location(div2, file$4, 86, 6, 2466);
				attr_dev(button0, "type", "submit");
				button0.disabled = /*isLoading*/ ctx[1];
				attr_dev(button0, "class", "w-full bg-red-600 hover:bg-red-700 text-white font-medium py-3 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center");
				add_location(button0, file$4, 110, 6, 3295);
				attr_dev(form, "class", "space-y-4");
				add_location(form, file$4, 63, 4, 1715);
				attr_dev(button1, "type", "button");
				attr_dev(button1, "class", "text-gray-400 hover:text-white transition-colors");
				add_location(button1, file$4, 124, 6, 3854);
				attr_dev(div3, "class", "mt-6 text-center");
				add_location(div3, file$4, 123, 4, 3817);
				attr_dev(path0, "fill-rule", "evenodd");
				attr_dev(path0, "d", "M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z");
				attr_dev(path0, "clip-rule", "evenodd");
				add_location(path0, file$4, 137, 10, 4333);
				attr_dev(svg0, "class", "w-5 h-5 text-red-500");
				attr_dev(svg0, "fill", "currentColor");
				attr_dev(svg0, "viewBox", "0 0 20 20");
				add_location(svg0, file$4, 136, 8, 4248);
				attr_dev(span1, "class", "text-sm");
				add_location(span1, file$4, 139, 8, 4543);
				attr_dev(div4, "class", "flex items-center space-x-3 text-gray-500");
				add_location(div4, file$4, 135, 6, 4184);
				attr_dev(path1, "fill-rule", "evenodd");
				attr_dev(path1, "d", "M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z");
				attr_dev(path1, "clip-rule", "evenodd");
				add_location(path1, file$4, 143, 10, 4774);
				attr_dev(svg1, "class", "w-5 h-5 text-red-500");
				attr_dev(svg1, "fill", "currentColor");
				attr_dev(svg1, "viewBox", "0 0 20 20");
				add_location(svg1, file$4, 142, 8, 4689);
				attr_dev(span2, "class", "text-sm");
				add_location(span2, file$4, 145, 8, 4984);
				attr_dev(div5, "class", "flex items-center space-x-3 text-gray-500");
				add_location(div5, file$4, 141, 6, 4625);
				attr_dev(path2, "fill-rule", "evenodd");
				attr_dev(path2, "d", "M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z");
				attr_dev(path2, "clip-rule", "evenodd");
				add_location(path2, file$4, 149, 10, 5220);
				attr_dev(svg2, "class", "w-5 h-5 text-red-500");
				attr_dev(svg2, "fill", "currentColor");
				attr_dev(svg2, "viewBox", "0 0 20 20");
				add_location(svg2, file$4, 148, 8, 5135);
				attr_dev(span3, "class", "text-sm");
				add_location(span3, file$4, 151, 8, 5430);
				attr_dev(div6, "class", "flex items-center space-x-3 text-gray-500");
				add_location(div6, file$4, 147, 6, 5071);
				attr_dev(div7, "class", "mt-12 space-y-3");
				add_location(div7, file$4, 134, 4, 4148);
				attr_dev(div8, "class", "w-full max-w-sm");
				add_location(div8, file$4, 54, 2, 1410);
				attr_dev(div9, "class", "min-h-screen flex flex-col items-center justify-center bg-gray-900 px-4");
				add_location(div9, file$4, 53, 0, 1322);
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				insert_dev(target, div9, anchor);
				append_dev(div9, div8);
				append_dev(div8, div0);
				append_dev(div0, span0);
				append_dev(div0, t1);
				append_dev(div0, h1);
				append_dev(div0, t3);
				append_dev(div0, p);
				append_dev(div8, t5);
				append_dev(div8, form);
				if (if_block0) if_block0.m(form, null);
				append_dev(form, t6);
				if (if_block1) if_block1.m(form, null);
				append_dev(form, t7);
				append_dev(form, div1);
				append_dev(div1, input0);
				set_input_value(input0, /*email*/ ctx[4]);
				append_dev(form, t8);
				append_dev(form, div2);
				append_dev(div2, input1);
				set_input_value(input1, /*password*/ ctx[5]);
				append_dev(form, t9);
				if (if_block2) if_block2.m(form, null);
				append_dev(form, t10);
				append_dev(form, button0);
				if (if_block3) if_block3.m(button0, null);
				append_dev(button0, t11);
				append_dev(button0, t12);
				append_dev(div8, t13);
				append_dev(div8, div3);
				append_dev(div3, button1);
				append_dev(button1, t14);
				append_dev(div8, t15);
				append_dev(div8, div7);
				append_dev(div7, div4);
				append_dev(div4, svg0);
				append_dev(svg0, path0);
				append_dev(div4, t16);
				append_dev(div4, span1);
				append_dev(div7, t18);
				append_dev(div7, div5);
				append_dev(div5, svg1);
				append_dev(svg1, path1);
				append_dev(div5, t19);
				append_dev(div5, span2);
				append_dev(div7, t21);
				append_dev(div7, div6);
				append_dev(div6, svg2);
				append_dev(svg2, path2);
				append_dev(div6, t22);
				append_dev(div6, span3);

				if (!mounted) {
					dispose = [
						listen_dev(input0, "input", /*input0_input_handler*/ ctx[9]),
						listen_dev(input1, "input", /*input1_input_handler*/ ctx[10]),
						listen_dev(form, "submit", prevent_default(/*handleSubmit*/ ctx[7]), false, true, false, false),
						listen_dev(button1, "click", /*switchMode*/ ctx[8], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, [dirty]) {
				if (/*error*/ ctx[2]) {
					if (if_block0) {
						if_block0.p(ctx, dirty);
					} else {
						if_block0 = create_if_block_3$3(ctx);
						if_block0.c();
						if_block0.m(form, t6);
					}
				} else if (if_block0) {
					if_block0.d(1);
					if_block0 = null;
				}

				if (/*success*/ ctx[3]) {
					if (if_block1) {
						if_block1.p(ctx, dirty);
					} else {
						if_block1 = create_if_block_2$3(ctx);
						if_block1.c();
						if_block1.m(form, t7);
					}
				} else if (if_block1) {
					if_block1.d(1);
					if_block1 = null;
				}

				if (dirty & /*email*/ 16 && input0.value !== /*email*/ ctx[4]) {
					set_input_value(input0, /*email*/ ctx[4]);
				}

				if (dirty & /*password*/ 32 && input1.value !== /*password*/ ctx[5]) {
					set_input_value(input1, /*password*/ ctx[5]);
				}

				if (/*mode*/ ctx[0] === 'register') {
					if (if_block2) {
						if_block2.p(ctx, dirty);
					} else {
						if_block2 = create_if_block_1$4(ctx);
						if_block2.c();
						if_block2.m(form, t10);
					}
				} else if (if_block2) {
					if_block2.d(1);
					if_block2 = null;
				}

				if (/*isLoading*/ ctx[1]) {
					if (if_block3) ; else {
						if_block3 = create_if_block$4(ctx);
						if_block3.c();
						if_block3.m(button0, t11);
					}
				} else if (if_block3) {
					if_block3.d(1);
					if_block3 = null;
				}

				if (dirty & /*mode*/ 1 && t12_value !== (t12_value = (/*mode*/ ctx[0] === 'login'
				? 'Sign in'
				: 'Create account') + "")) set_data_dev(t12, t12_value);

				if (dirty & /*isLoading*/ 2) {
					prop_dev(button0, "disabled", /*isLoading*/ ctx[1]);
				}

				if (dirty & /*mode*/ 1 && t14_value !== (t14_value = (/*mode*/ ctx[0] === 'login'
				? "Don't have an account? Sign up"
				: 'Already have an account? Sign in') + "")) set_data_dev(t14, t14_value);
			},
			i: noop,
			o: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div9);
				}

				if (if_block0) if_block0.d();
				if (if_block1) if_block1.d();
				if (if_block2) if_block2.d();
				if (if_block3) if_block3.d();
				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_fragment$4.name,
			type: "component",
			source: "",
			ctx
		});

		return block;
	}

	function instance$4($$self, $$props, $$invalidate) {
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('Login', slots, []);
		let mode = 'login';
		let isLoading = false;
		let error = '';
		let success = '';

		// Form fields
		let email = '';

		let password = '';
		let confirmPassword = '';

		async function handleSubmit() {
			$$invalidate(2, error = '');
			$$invalidate(3, success = '');
			$$invalidate(1, isLoading = true);

			try {
				if (mode === 'login') {
					const result = await authActions.login(email, password);

					if (!result.success) {
						$$invalidate(2, error = result.message || 'Login failed');
					}
				} else {
					if (password !== confirmPassword) {
						$$invalidate(2, error = 'Passwords do not match');
						$$invalidate(1, isLoading = false);
						return;
					}

					const result = await authActions.register(email, password, confirmPassword, true);

					if (result.success) {
						$$invalidate(3, success = 'Account created!');
					} else {
						$$invalidate(2, error = result.message || 'Registration failed');
					}
				}
			} catch(err) {
				$$invalidate(2, error = 'Something went wrong. Please try again.');
			} finally {
				$$invalidate(1, isLoading = false);
			}
		}

		function switchMode() {
			$$invalidate(0, mode = mode === 'login' ? 'register' : 'login');
			$$invalidate(2, error = '');
			$$invalidate(3, success = '');
			$$invalidate(5, password = '');
			$$invalidate(6, confirmPassword = '');
		}

		const writable_props = [];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<Login> was created with unknown prop '${key}'`);
		});

		function input0_input_handler() {
			email = this.value;
			$$invalidate(4, email);
		}

		function input1_input_handler() {
			password = this.value;
			$$invalidate(5, password);
		}

		function input_input_handler() {
			confirmPassword = this.value;
			$$invalidate(6, confirmPassword);
		}

		$$self.$capture_state = () => ({
			authActions,
			mode,
			isLoading,
			error,
			success,
			email,
			password,
			confirmPassword,
			handleSubmit,
			switchMode
		});

		$$self.$inject_state = $$props => {
			if ('mode' in $$props) $$invalidate(0, mode = $$props.mode);
			if ('isLoading' in $$props) $$invalidate(1, isLoading = $$props.isLoading);
			if ('error' in $$props) $$invalidate(2, error = $$props.error);
			if ('success' in $$props) $$invalidate(3, success = $$props.success);
			if ('email' in $$props) $$invalidate(4, email = $$props.email);
			if ('password' in $$props) $$invalidate(5, password = $$props.password);
			if ('confirmPassword' in $$props) $$invalidate(6, confirmPassword = $$props.confirmPassword);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		return [
			mode,
			isLoading,
			error,
			success,
			email,
			password,
			confirmPassword,
			handleSubmit,
			switchMode,
			input0_input_handler,
			input1_input_handler,
			input_input_handler
		];
	}

	class Login extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance$4, create_fragment$4, safe_not_equal, {});

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "Login",
				options,
				id: create_fragment$4.name
			});
		}
	}

	/* src/lib/components/Home.svelte generated by Svelte v4.2.20 */

	const { console: console_1$2 } = globals;
	const file$3 = "src/lib/components/Home.svelte";

	function get_each_context(ctx, list, i) {
		const child_ctx = ctx.slice();
		child_ctx[25] = list[i];
		return child_ctx;
	}

	function get_each_context_2(ctx, list, i) {
		const child_ctx = ctx.slice();
		child_ctx[31] = list[i];
		return child_ctx;
	}

	function get_each_context_1(ctx, list, i) {
		const child_ctx = ctx.slice();
		child_ctx[28] = list[i];
		return child_ctx;
	}

	function get_each_context_4(ctx, list, i) {
		const child_ctx = ctx.slice();
		child_ctx[36] = list[i];
		return child_ctx;
	}

	function get_each_context_3(ctx, list, i) {
		const child_ctx = ctx.slice();
		child_ctx[28] = list[i];
		return child_ctx;
	}

	function get_each_context_5(ctx, list, i) {
		const child_ctx = ctx.slice();
		child_ctx[39] = list[i];
		return child_ctx;
	}

	// (242:8) {#if isSearching}
	function create_if_block_11(ctx) {
		let div1;
		let div0;

		const block = {
			c: function create() {
				div1 = element("div");
				div0 = element("div");
				attr_dev(div0, "class", "w-5 h-5 border-2 border-red-500 border-t-transparent rounded-full animate-spin");
				add_location(div0, file$3, 288, 12, 8420);
				attr_dev(div1, "class", "absolute right-4 top-1/2 -translate-y-1/2");
				add_location(div1, file$3, 287, 10, 8352);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div1, anchor);
				append_dev(div1, div0);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div1);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_11.name,
			type: "if",
			source: "(242:8) {#if isSearching}",
			ctx
		});

		return block;
	}

	// (250:6) {#if searchResults.length > 0}
	function create_if_block_6$1(ctx) {
		let div;
		let each_value_5 = ensure_array_like_dev(/*searchResults*/ ctx[1]);
		let each_blocks = [];

		for (let i = 0; i < each_value_5.length; i += 1) {
			each_blocks[i] = create_each_block_5(get_each_context_5(ctx, each_value_5, i));
		}

		const block = {
			c: function create() {
				div = element("div");

				for (let i = 0; i < each_blocks.length; i += 1) {
					each_blocks[i].c();
				}

				attr_dev(div, "class", "mt-3 bg-gray-800 rounded-xl border border-gray-600 overflow-hidden");
				add_location(div, file$3, 295, 8, 8639);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);

				for (let i = 0; i < each_blocks.length; i += 1) {
					if (each_blocks[i]) {
						each_blocks[i].m(div, null);
					}
				}
			},
			p: function update(ctx, dirty) {
				if (dirty[0] & /*blockingArtistId, searchResults, unblockArtist, blockArtist*/ 12546) {
					each_value_5 = ensure_array_like_dev(/*searchResults*/ ctx[1]);
					let i;

					for (i = 0; i < each_value_5.length; i += 1) {
						const child_ctx = get_each_context_5(ctx, each_value_5, i);

						if (each_blocks[i]) {
							each_blocks[i].p(child_ctx, dirty);
						} else {
							each_blocks[i] = create_each_block_5(child_ctx);
							each_blocks[i].c();
							each_blocks[i].m(div, null);
						}
					}

					for (; i < each_blocks.length; i += 1) {
						each_blocks[i].d(1);
					}

					each_blocks.length = each_value_5.length;
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}

				destroy_each(each_blocks, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_6$1.name,
			type: "if",
			source: "(250:6) {#if searchResults.length > 0}",
			ctx
		});

		return block;
	}

	// (257:16) {:else}
	function create_else_block_4$1(ctx) {
		let div;
		let span;
		let t_value = /*artist*/ ctx[39].name.charAt(0) + "";
		let t;

		const block = {
			c: function create() {
				div = element("div");
				span = element("span");
				t = text(t_value);
				attr_dev(span, "class", "text-xl");
				add_location(span, file$3, 303, 20, 9233);
				attr_dev(div, "class", "w-12 h-12 rounded-full bg-gray-700 flex items-center justify-center");
				add_location(div, file$3, 302, 18, 9131);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, span);
				append_dev(span, t);
			},
			p: function update(ctx, dirty) {
				if (dirty[0] & /*searchResults*/ 2 && t_value !== (t_value = /*artist*/ ctx[39].name.charAt(0) + "")) set_data_dev(t, t_value);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block_4$1.name,
			type: "else",
			source: "(257:16) {:else}",
			ctx
		});

		return block;
	}

	// (255:16) {#if artist.image_url}
	function create_if_block_10(ctx) {
		let img;
		let img_src_value;
		let img_alt_value;

		const block = {
			c: function create() {
				img = element("img");
				if (!src_url_equal(img.src, img_src_value = /*artist*/ ctx[39].image_url)) attr_dev(img, "src", img_src_value);
				attr_dev(img, "alt", img_alt_value = /*artist*/ ctx[39].name);
				attr_dev(img, "class", "w-12 h-12 rounded-full object-cover");
				add_location(img, file$3, 300, 18, 8996);
			},
			m: function mount(target, anchor) {
				insert_dev(target, img, anchor);
			},
			p: function update(ctx, dirty) {
				if (dirty[0] & /*searchResults*/ 2 && !src_url_equal(img.src, img_src_value = /*artist*/ ctx[39].image_url)) {
					attr_dev(img, "src", img_src_value);
				}

				if (dirty[0] & /*searchResults*/ 2 && img_alt_value !== (img_alt_value = /*artist*/ ctx[39].name)) {
					attr_dev(img, "alt", img_alt_value);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(img);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_10.name,
			type: "if",
			source: "(255:16) {#if artist.image_url}",
			ctx
		});

		return block;
	}

	// (264:18) {#if artist.genres && artist.genres.length > 0}
	function create_if_block_9(ctx) {
		let p;
		let t_value = /*artist*/ ctx[39].genres.slice(0, 2).join(', ') + "";
		let t;

		const block = {
			c: function create() {
				p = element("p");
				t = text(t_value);
				attr_dev(p, "class", "text-sm text-gray-300");
				add_location(p, file$3, 309, 20, 9500);
			},
			m: function mount(target, anchor) {
				insert_dev(target, p, anchor);
				append_dev(p, t);
			},
			p: function update(ctx, dirty) {
				if (dirty[0] & /*searchResults*/ 2 && t_value !== (t_value = /*artist*/ ctx[39].genres.slice(0, 2).join(', ') + "")) set_data_dev(t, t_value);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(p);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_9.name,
			type: "if",
			source: "(264:18) {#if artist.genres && artist.genres.length > 0}",
			ctx
		});

		return block;
	}

	// (282:16) {:else}
	function create_else_block_3$1(ctx) {
		let t;

		const block = {
			c: function create() {
				t = text("Block");
			},
			m: function mount(target, anchor) {
				insert_dev(target, t, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(t);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block_3$1.name,
			type: "else",
			source: "(282:16) {:else}",
			ctx
		});

		return block;
	}

	// (280:41) 
	function create_if_block_8(ctx) {
		let t;

		const block = {
			c: function create() {
				t = text("Blocked");
			},
			m: function mount(target, anchor) {
				insert_dev(target, t, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(t);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_8.name,
			type: "if",
			source: "(280:41) ",
			ctx
		});

		return block;
	}

	// (278:16) {#if blockingArtistId === artist.id}
	function create_if_block_7$1(ctx) {
		let span;

		const block = {
			c: function create() {
				span = element("span");
				attr_dev(span, "class", "inline-block w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin");
				add_location(span, file$3, 323, 18, 10183);
			},
			m: function mount(target, anchor) {
				insert_dev(target, span, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(span);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_7$1.name,
			type: "if",
			source: "(278:16) {#if blockingArtistId === artist.id}",
			ctx
		});

		return block;
	}

	// (252:10) {#each searchResults as artist}
	function create_each_block_5(ctx) {
		let div2;
		let div1;
		let t0;
		let div0;
		let p;
		let t1_value = /*artist*/ ctx[39].name + "";
		let t1;
		let t2;
		let t3;
		let button;
		let button_disabled_value;
		let button_class_value;
		let t4;
		let mounted;
		let dispose;

		function select_block_type(ctx, dirty) {
			if (/*artist*/ ctx[39].image_url) return create_if_block_10;
			return create_else_block_4$1;
		}

		let current_block_type = select_block_type(ctx);
		let if_block0 = current_block_type(ctx);
		let if_block1 = /*artist*/ ctx[39].genres && /*artist*/ ctx[39].genres.length > 0 && create_if_block_9(ctx);

		function select_block_type_1(ctx, dirty) {
			if (/*blockingArtistId*/ ctx[8] === /*artist*/ ctx[39].id) return create_if_block_7$1;
			if (/*artist*/ ctx[39].blocked) return create_if_block_8;
			return create_else_block_3$1;
		}

		let current_block_type_1 = select_block_type_1(ctx);
		let if_block2 = current_block_type_1(ctx);

		function click_handler_1() {
			return /*click_handler_1*/ ctx[17](/*artist*/ ctx[39]);
		}

		const block = {
			c: function create() {
				div2 = element("div");
				div1 = element("div");
				if_block0.c();
				t0 = space();
				div0 = element("div");
				p = element("p");
				t1 = text(t1_value);
				t2 = space();
				if (if_block1) if_block1.c();
				t3 = space();
				button = element("button");
				if_block2.c();
				t4 = space();
				attr_dev(p, "class", "font-medium");
				add_location(p, file$3, 307, 18, 9373);
				add_location(div0, file$3, 306, 16, 9349);
				attr_dev(div1, "class", "flex items-center space-x-4");
				add_location(div1, file$3, 298, 14, 8897);
				button.disabled = button_disabled_value = /*blockingArtistId*/ ctx[8] === /*artist*/ ctx[39].id;

				attr_dev(button, "class", button_class_value = "px-4 py-2 rounded-lg font-medium transition-all " + (/*artist*/ ctx[39].blocked
				? 'bg-gray-600 text-gray-200 hover:bg-gray-500'
				: 'bg-rose-600 text-white hover:bg-rose-700') + " disabled:opacity-50");

				add_location(button, file$3, 313, 14, 9658);
				attr_dev(div2, "class", "flex items-center justify-between p-4 hover:bg-gray-700 border-b border-gray-600 last:border-0");
				add_location(div2, file$3, 297, 12, 8774);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div2, anchor);
				append_dev(div2, div1);
				if_block0.m(div1, null);
				append_dev(div1, t0);
				append_dev(div1, div0);
				append_dev(div0, p);
				append_dev(p, t1);
				append_dev(div0, t2);
				if (if_block1) if_block1.m(div0, null);
				append_dev(div2, t3);
				append_dev(div2, button);
				if_block2.m(button, null);
				append_dev(div2, t4);

				if (!mounted) {
					dispose = listen_dev(button, "click", click_handler_1, false, false, false, false);
					mounted = true;
				}
			},
			p: function update(new_ctx, dirty) {
				ctx = new_ctx;

				if (current_block_type === (current_block_type = select_block_type(ctx)) && if_block0) {
					if_block0.p(ctx, dirty);
				} else {
					if_block0.d(1);
					if_block0 = current_block_type(ctx);

					if (if_block0) {
						if_block0.c();
						if_block0.m(div1, t0);
					}
				}

				if (dirty[0] & /*searchResults*/ 2 && t1_value !== (t1_value = /*artist*/ ctx[39].name + "")) set_data_dev(t1, t1_value);

				if (/*artist*/ ctx[39].genres && /*artist*/ ctx[39].genres.length > 0) {
					if (if_block1) {
						if_block1.p(ctx, dirty);
					} else {
						if_block1 = create_if_block_9(ctx);
						if_block1.c();
						if_block1.m(div0, null);
					}
				} else if (if_block1) {
					if_block1.d(1);
					if_block1 = null;
				}

				if (current_block_type_1 !== (current_block_type_1 = select_block_type_1(ctx))) {
					if_block2.d(1);
					if_block2 = current_block_type_1(ctx);

					if (if_block2) {
						if_block2.c();
						if_block2.m(button, null);
					}
				}

				if (dirty[0] & /*blockingArtistId, searchResults*/ 258 && button_disabled_value !== (button_disabled_value = /*blockingArtistId*/ ctx[8] === /*artist*/ ctx[39].id)) {
					prop_dev(button, "disabled", button_disabled_value);
				}

				if (dirty[0] & /*searchResults*/ 2 && button_class_value !== (button_class_value = "px-4 py-2 rounded-lg font-medium transition-all " + (/*artist*/ ctx[39].blocked
				? 'bg-gray-600 text-gray-200 hover:bg-gray-500'
				: 'bg-rose-600 text-white hover:bg-rose-700') + " disabled:opacity-50")) {
					attr_dev(button, "class", button_class_value);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div2);
				}

				if_block0.d();
				if (if_block1) if_block1.d();
				if_block2.d();
				mounted = false;
				dispose();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_each_block_5.name,
			type: "each",
			source: "(252:10) {#each searchResults as artist}",
			ctx
		});

		return block;
	}

	// (310:8) {:else}
	function create_else_block_2$1(ctx) {
		let div;
		let each_value_4 = ensure_array_like_dev(/*newsFeed*/ ctx[3]);
		let each_blocks = [];

		for (let i = 0; i < each_value_4.length; i += 1) {
			each_blocks[i] = create_each_block_4(get_each_context_4(ctx, each_value_4, i));
		}

		const block = {
			c: function create() {
				div = element("div");

				for (let i = 0; i < each_blocks.length; i += 1) {
					each_blocks[i].c();
				}

				attr_dev(div, "class", "space-y-3");
				add_location(div, file$3, 355, 10, 11253);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);

				for (let i = 0; i < each_blocks.length; i += 1) {
					if (each_blocks[i]) {
						each_blocks[i].m(div, null);
					}
				}
			},
			p: function update(ctx, dirty) {
				if (dirty[0] & /*newsFeed, categoryColors, categoryLabels*/ 1544) {
					each_value_4 = ensure_array_like_dev(/*newsFeed*/ ctx[3]);
					let i;

					for (i = 0; i < each_value_4.length; i += 1) {
						const child_ctx = get_each_context_4(ctx, each_value_4, i);

						if (each_blocks[i]) {
							each_blocks[i].p(child_ctx, dirty);
						} else {
							each_blocks[i] = create_each_block_4(child_ctx);
							each_blocks[i].c();
							each_blocks[i].m(div, null);
						}
					}

					for (; i < each_blocks.length; i += 1) {
						each_blocks[i].d(1);
					}

					each_blocks.length = each_value_4.length;
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}

				destroy_each(each_blocks, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block_2$1.name,
			type: "else",
			source: "(310:8) {:else}",
			ctx
		});

		return block;
	}

	// (301:8) {#if isLoadingNews}
	function create_if_block_5$1(ctx) {
		let div;
		let each_value_3 = ensure_array_like_dev([1, 2, 3]);
		let each_blocks = [];

		for (let i = 0; i < 3; i += 1) {
			each_blocks[i] = create_each_block_3(get_each_context_3(ctx, each_value_3, i));
		}

		const block = {
			c: function create() {
				div = element("div");

				for (let i = 0; i < 3; i += 1) {
					each_blocks[i].c();
				}

				attr_dev(div, "class", "space-y-3");
				add_location(div, file$3, 346, 10, 10904);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);

				for (let i = 0; i < 3; i += 1) {
					if (each_blocks[i]) {
						each_blocks[i].m(div, null);
					}
				}
			},
			p: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}

				destroy_each(each_blocks, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_5$1.name,
			type: "if",
			source: "(301:8) {#if isLoadingNews}",
			ctx
		});

		return block;
	}

	// (312:12) {#each newsFeed as item}
	function create_each_block_4(ctx) {
		let div3;
		let div2;
		let div1;
		let div0;
		let span0;
		let t0_value = (/*categoryLabels*/ ctx[10][/*item*/ ctx[36].category] || /*item*/ ctx[36].category) + "";
		let t0;
		let span0_class_value;
		let t1;
		let span1;
		let t2_value = formatTimeAgo(/*item*/ ctx[36].timestamp) + "";
		let t2;
		let t3;
		let p0;
		let t4_value = /*item*/ ctx[36].artist_name + "";
		let t4;
		let t5;
		let p1;
		let t6_value = /*item*/ ctx[36].headline + "";
		let t6;
		let t7;
		let a;
		let t8_value = /*item*/ ctx[36].source + "";
		let t8;
		let a_href_value;
		let t9;
		let button;
		let t11;

		const block = {
			c: function create() {
				div3 = element("div");
				div2 = element("div");
				div1 = element("div");
				div0 = element("div");
				span0 = element("span");
				t0 = text(t0_value);
				t1 = space();
				span1 = element("span");
				t2 = text(t2_value);
				t3 = space();
				p0 = element("p");
				t4 = text(t4_value);
				t5 = space();
				p1 = element("p");
				t6 = text(t6_value);
				t7 = space();
				a = element("a");
				t8 = text(t8_value);
				t9 = space();
				button = element("button");
				button.textContent = "Block";
				t11 = space();
				attr_dev(span0, "class", span0_class_value = "px-2 py-0.5 rounded text-xs font-medium " + (/*categoryColors*/ ctx[9][/*item*/ ctx[36].category] || 'bg-gray-500'));
				add_location(span0, file$3, 361, 22, 11623);
				attr_dev(span1, "class", "text-xs text-gray-400");
				add_location(span1, file$3, 364, 22, 11852);
				attr_dev(div0, "class", "flex items-center space-x-2 mb-1");
				add_location(div0, file$3, 360, 20, 11554);
				attr_dev(p0, "class", "font-medium");
				add_location(p0, file$3, 366, 20, 11974);
				attr_dev(p1, "class", "text-sm text-gray-300 mt-1");
				add_location(p1, file$3, 367, 20, 12040);
				attr_dev(a, "href", a_href_value = /*item*/ ctx[36].source_url);
				attr_dev(a, "target", "_blank");
				attr_dev(a, "rel", "noopener");
				attr_dev(a, "class", "text-xs text-blue-400 hover:underline mt-2 inline-block");
				add_location(a, file$3, 368, 20, 12118);
				attr_dev(div1, "class", "flex-1");
				add_location(div1, file$3, 359, 18, 11513);
				attr_dev(button, "class", "ml-4 px-3 py-1.5 bg-rose-600 hover:bg-rose-700 rounded-lg text-sm font-medium transition-colors");
				add_location(button, file$3, 372, 18, 12344);
				attr_dev(div2, "class", "flex items-start justify-between");
				add_location(div2, file$3, 358, 16, 11448);
				attr_dev(div3, "class", "bg-gray-800 rounded-xl p-4 border border-gray-600 hover:border-gray-500 transition-colors");
				add_location(div3, file$3, 357, 14, 11328);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div3, anchor);
				append_dev(div3, div2);
				append_dev(div2, div1);
				append_dev(div1, div0);
				append_dev(div0, span0);
				append_dev(span0, t0);
				append_dev(div0, t1);
				append_dev(div0, span1);
				append_dev(span1, t2);
				append_dev(div1, t3);
				append_dev(div1, p0);
				append_dev(p0, t4);
				append_dev(div1, t5);
				append_dev(div1, p1);
				append_dev(p1, t6);
				append_dev(div1, t7);
				append_dev(div1, a);
				append_dev(a, t8);
				append_dev(div2, t9);
				append_dev(div2, button);
				append_dev(div3, t11);
			},
			p: function update(ctx, dirty) {
				if (dirty[0] & /*newsFeed*/ 8 && t0_value !== (t0_value = (/*categoryLabels*/ ctx[10][/*item*/ ctx[36].category] || /*item*/ ctx[36].category) + "")) set_data_dev(t0, t0_value);

				if (dirty[0] & /*newsFeed*/ 8 && span0_class_value !== (span0_class_value = "px-2 py-0.5 rounded text-xs font-medium " + (/*categoryColors*/ ctx[9][/*item*/ ctx[36].category] || 'bg-gray-500'))) {
					attr_dev(span0, "class", span0_class_value);
				}

				if (dirty[0] & /*newsFeed*/ 8 && t2_value !== (t2_value = formatTimeAgo(/*item*/ ctx[36].timestamp) + "")) set_data_dev(t2, t2_value);
				if (dirty[0] & /*newsFeed*/ 8 && t4_value !== (t4_value = /*item*/ ctx[36].artist_name + "")) set_data_dev(t4, t4_value);
				if (dirty[0] & /*newsFeed*/ 8 && t6_value !== (t6_value = /*item*/ ctx[36].headline + "")) set_data_dev(t6, t6_value);
				if (dirty[0] & /*newsFeed*/ 8 && t8_value !== (t8_value = /*item*/ ctx[36].source + "")) set_data_dev(t8, t8_value);

				if (dirty[0] & /*newsFeed*/ 8 && a_href_value !== (a_href_value = /*item*/ ctx[36].source_url)) {
					attr_dev(a, "href", a_href_value);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div3);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_each_block_4.name,
			type: "each",
			source: "(312:12) {#each newsFeed as item}",
			ctx
		});

		return block;
	}

	// (303:12) {#each [1, 2, 3] as _}
	function create_each_block_3(ctx) {
		let div2;
		let div0;
		let t0;
		let div1;
		let t1;

		const block = {
			c: function create() {
				div2 = element("div");
				div0 = element("div");
				t0 = space();
				div1 = element("div");
				t1 = space();
				attr_dev(div0, "class", "h-4 bg-gray-700 rounded w-3/4 mb-2");
				add_location(div0, file$3, 349, 16, 11048);
				attr_dev(div1, "class", "h-3 bg-gray-700 rounded w-1/2");
				add_location(div1, file$3, 350, 16, 11119);
				attr_dev(div2, "class", "bg-gray-800 rounded-xl p-4 animate-pulse");
				add_location(div2, file$3, 348, 14, 10977);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div2, anchor);
				append_dev(div2, div0);
				append_dev(div2, t0);
				append_dev(div2, div1);
				append_dev(div2, t1);
			},
			p: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div2);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_each_block_3.name,
			type: "each",
			source: "(303:12) {#each [1, 2, 3] as _}",
			ctx
		});

		return block;
	}

	// (352:10) {:else}
	function create_else_block_1$2(ctx) {
		let div;
		let each_value_2 = ensure_array_like_dev(/*categoryLists*/ ctx[5]);
		let each_blocks = [];

		for (let i = 0; i < each_value_2.length; i += 1) {
			each_blocks[i] = create_each_block_2(get_each_context_2(ctx, each_value_2, i));
		}

		const block = {
			c: function create() {
				div = element("div");

				for (let i = 0; i < each_blocks.length; i += 1) {
					each_blocks[i].c();
				}

				attr_dev(div, "class", "space-y-2");
				add_location(div, file$3, 397, 12, 13145);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);

				for (let i = 0; i < each_blocks.length; i += 1) {
					if (each_blocks[i]) {
						each_blocks[i].m(div, null);
					}
				}
			},
			p: function update(ctx, dirty) {
				if (dirty[0] & /*toggleCategory, categoryLists*/ 16416) {
					each_value_2 = ensure_array_like_dev(/*categoryLists*/ ctx[5]);
					let i;

					for (i = 0; i < each_value_2.length; i += 1) {
						const child_ctx = get_each_context_2(ctx, each_value_2, i);

						if (each_blocks[i]) {
							each_blocks[i].p(child_ctx, dirty);
						} else {
							each_blocks[i] = create_each_block_2(child_ctx);
							each_blocks[i].c();
							each_blocks[i].m(div, null);
						}
					}

					for (; i < each_blocks.length; i += 1) {
						each_blocks[i].d(1);
					}

					each_blocks.length = each_value_2.length;
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}

				destroy_each(each_blocks, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block_1$2.name,
			type: "else",
			source: "(352:10) {:else}",
			ctx
		});

		return block;
	}

	// (344:10) {#if isLoadingCategories}
	function create_if_block_3$2(ctx) {
		let div;
		let each_value_1 = ensure_array_like_dev([1, 2, 3, 4]);
		let each_blocks = [];

		for (let i = 0; i < 4; i += 1) {
			each_blocks[i] = create_each_block_1(get_each_context_1(ctx, each_value_1, i));
		}

		const block = {
			c: function create() {
				div = element("div");

				for (let i = 0; i < 4; i += 1) {
					each_blocks[i].c();
				}

				attr_dev(div, "class", "space-y-2");
				add_location(div, file$3, 389, 12, 12848);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);

				for (let i = 0; i < 4; i += 1) {
					if (each_blocks[i]) {
						each_blocks[i].m(div, null);
					}
				}
			},
			p: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}

				destroy_each(each_blocks, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_3$2.name,
			type: "if",
			source: "(344:10) {#if isLoadingCategories}",
			ctx
		});

		return block;
	}

	// (372:22) {#if category.subscribed}
	function create_if_block_4$2(ctx) {
		let svg;
		let path;

		const block = {
			c: function create() {
				svg = svg_element("svg");
				path = svg_element("path");
				attr_dev(path, "fill-rule", "evenodd");
				attr_dev(path, "d", "M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z");
				attr_dev(path, "clip-rule", "evenodd");
				add_location(path, file$3, 418, 26, 14390);
				attr_dev(svg, "class", "w-3 h-3 text-white");
				attr_dev(svg, "fill", "currentColor");
				attr_dev(svg, "viewBox", "0 0 20 20");
				add_location(svg, file$3, 417, 24, 14291);
			},
			m: function mount(target, anchor) {
				insert_dev(target, svg, anchor);
				append_dev(svg, path);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(svg);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_4$2.name,
			type: "if",
			source: "(372:22) {#if category.subscribed}",
			ctx
		});

		return block;
	}

	// (354:14) {#each categoryLists as category}
	function create_each_block_2(ctx) {
		let button;
		let div4;
		let div2;
		let div0;
		let div0_class_value;
		let t0;
		let div1;
		let p0;
		let t1_value = /*category*/ ctx[31].name + "";
		let t1;
		let t2;
		let p1;
		let t3_value = /*category*/ ctx[31].artist_count + "";
		let t3;
		let t4;
		let t5;
		let div3;
		let div3_class_value;
		let t6;
		let mounted;
		let dispose;
		let if_block = /*category*/ ctx[31].subscribed && create_if_block_4$2(ctx);

		function click_handler_2() {
			return /*click_handler_2*/ ctx[18](/*category*/ ctx[31]);
		}

		const block = {
			c: function create() {
				button = element("button");
				div4 = element("div");
				div2 = element("div");
				div0 = element("div");
				t0 = space();
				div1 = element("div");
				p0 = element("p");
				t1 = text(t1_value);
				t2 = space();
				p1 = element("p");
				t3 = text(t3_value);
				t4 = text(" artists");
				t5 = space();
				div3 = element("div");
				if (if_block) if_block.c();
				t6 = space();
				attr_dev(div0, "class", div0_class_value = "w-3 h-3 rounded-full " + /*category*/ ctx[31].color);
				add_location(div0, file$3, 405, 22, 13605);
				attr_dev(p0, "class", "font-medium text-sm");
				add_location(p0, file$3, 407, 24, 13715);
				attr_dev(p1, "class", "text-xs text-gray-400");
				add_location(p1, file$3, 408, 24, 13790);
				add_location(div1, file$3, 406, 22, 13685);
				attr_dev(div2, "class", "flex items-center space-x-3");
				add_location(div2, file$3, 404, 20, 13541);

				attr_dev(div3, "class", div3_class_value = "w-5 h-5 rounded border-2 flex items-center justify-center transition-colors " + (/*category*/ ctx[31].subscribed
				? 'bg-rose-500 border-rose-500'
				: 'border-gray-500 group-hover:border-gray-400'));

				add_location(div3, file$3, 411, 20, 13935);
				attr_dev(div4, "class", "flex items-center justify-between");
				add_location(div4, file$3, 403, 18, 13473);
				attr_dev(button, "class", "w-full bg-gray-800 rounded-lg p-3 border border-gray-600 hover:border-gray-500 transition-all text-left group");
				add_location(button, file$3, 399, 16, 13233);
			},
			m: function mount(target, anchor) {
				insert_dev(target, button, anchor);
				append_dev(button, div4);
				append_dev(div4, div2);
				append_dev(div2, div0);
				append_dev(div2, t0);
				append_dev(div2, div1);
				append_dev(div1, p0);
				append_dev(p0, t1);
				append_dev(div1, t2);
				append_dev(div1, p1);
				append_dev(p1, t3);
				append_dev(p1, t4);
				append_dev(div4, t5);
				append_dev(div4, div3);
				if (if_block) if_block.m(div3, null);
				append_dev(button, t6);

				if (!mounted) {
					dispose = listen_dev(button, "click", click_handler_2, false, false, false, false);
					mounted = true;
				}
			},
			p: function update(new_ctx, dirty) {
				ctx = new_ctx;

				if (dirty[0] & /*categoryLists*/ 32 && div0_class_value !== (div0_class_value = "w-3 h-3 rounded-full " + /*category*/ ctx[31].color)) {
					attr_dev(div0, "class", div0_class_value);
				}

				if (dirty[0] & /*categoryLists*/ 32 && t1_value !== (t1_value = /*category*/ ctx[31].name + "")) set_data_dev(t1, t1_value);
				if (dirty[0] & /*categoryLists*/ 32 && t3_value !== (t3_value = /*category*/ ctx[31].artist_count + "")) set_data_dev(t3, t3_value);

				if (/*category*/ ctx[31].subscribed) {
					if (if_block) ; else {
						if_block = create_if_block_4$2(ctx);
						if_block.c();
						if_block.m(div3, null);
					}
				} else if (if_block) {
					if_block.d(1);
					if_block = null;
				}

				if (dirty[0] & /*categoryLists*/ 32 && div3_class_value !== (div3_class_value = "w-5 h-5 rounded border-2 flex items-center justify-center transition-colors " + (/*category*/ ctx[31].subscribed
				? 'bg-rose-500 border-rose-500'
				: 'border-gray-500 group-hover:border-gray-400'))) {
					attr_dev(div3, "class", div3_class_value);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(button);
				}

				if (if_block) if_block.d();
				mounted = false;
				dispose();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_each_block_2.name,
			type: "each",
			source: "(354:14) {#each categoryLists as category}",
			ctx
		});

		return block;
	}

	// (346:14) {#each [1, 2, 3, 4] as _}
	function create_each_block_1(ctx) {
		let div1;
		let div0;
		let t;

		const block = {
			c: function create() {
				div1 = element("div");
				div0 = element("div");
				t = space();
				attr_dev(div0, "class", "h-4 bg-gray-700 rounded w-2/3");
				add_location(div0, file$3, 392, 18, 13001);
				attr_dev(div1, "class", "bg-gray-800 rounded-lg p-3 animate-pulse");
				add_location(div1, file$3, 391, 16, 12928);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div1, anchor);
				append_dev(div1, div0);
				append_dev(div1, t);
			},
			p: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div1);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_each_block_1.name,
			type: "each",
			source: "(346:14) {#each [1, 2, 3, 4] as _}",
			ctx
		});

		return block;
	}

	// (399:65) 
	function create_if_block_2$2(ctx) {
		let div;
		let svg;
		let path;

		const block = {
			c: function create() {
				div = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				attr_dev(path, "d", "M23.994 6.124a9.23 9.23 0 00-.24-2.19c-.317-1.31-1.062-2.31-2.18-3.043a5.022 5.022 0 00-1.877-.726 10.496 10.496 0 00-1.564-.15c-.04-.003-.083-.01-.124-.013H5.986c-.152.01-.303.017-.455.026-.747.043-1.49.123-2.193.4-1.336.53-2.3 1.452-2.865 2.78-.192.448-.292.925-.363 1.408-.056.392-.088.785-.1 1.18 0 .032-.007.062-.01.093v12.223c.01.14.017.283.027.424.05.815.154 1.624.497 2.373.65 1.42 1.738 2.353 3.234 2.801.42.127.856.187 1.293.228.555.053 1.11.06 1.667.06h11.03a12.5 12.5 0 001.57-.1c.822-.106 1.596-.35 2.295-.81a5.046 5.046 0 001.88-2.207c.186-.42.293-.87.37-1.324.113-.675.138-1.358.137-2.04-.002-3.8 0-7.595-.003-11.393zm-6.423 3.99v5.712c0 .417-.058.827-.244 1.206-.29.59-.76.962-1.388 1.14-.35.1-.706.157-1.07.173-.95.042-1.785-.476-2.144-1.32-.238-.56-.223-1.136-.017-1.7.303-.825.96-1.277 1.743-1.49.294-.08.595-.13.893-.18.323-.054.65-.1.973-.157.274-.048.47-.202.53-.486a.707.707 0 00.017-.146c.002-1.633.002-3.265.002-4.898v-.07l-.06-.01c-2.097.4-4.194.8-6.29 1.202-.014.002-.032.014-.037.026-.006.016-.003.037-.003.056v7.36c0 .418-.052.832-.227 1.218-.282.622-.76 1.02-1.416 1.207-.313.09-.634.138-.96.166-.906.08-1.732-.4-2.134-1.203-.268-.534-.278-1.1-.096-1.66.267-.817.864-1.304 1.64-1.55.376-.12.763-.185 1.148-.25.278-.047.558-.088.832-.145.317-.065.522-.25.58-.574a.504.504 0 00.007-.115v-8.41c0-.25.042-.493.15-.72.183-.385.486-.62.882-.728.17-.047.346-.073.522-.11 2.55-.526 5.1-1.05 7.65-1.573.093-.02.19-.03.285-.03.316.004.528.2.613.5.032.113.044.233.044.35v5.9z");
				add_location(path, file$3, 446, 26, 16444);
				attr_dev(svg, "class", "w-5 h-5 text-white");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "fill", "currentColor");
				add_location(svg, file$3, 445, 24, 16345);
				attr_dev(div, "class", "w-8 h-8 rounded-full bg-gradient-to-br from-red-500 to-pink-500 flex items-center justify-center");
				add_location(div, file$3, 444, 22, 16210);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, svg);
				append_dev(svg, path);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_2$2.name,
			type: "if",
			source: "(399:65) ",
			ctx
		});

		return block;
	}

	// (393:20) {#if service.provider === 'spotify'}
	function create_if_block_1$3(ctx) {
		let div;
		let svg;
		let path;

		const block = {
			c: function create() {
				div = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				attr_dev(path, "d", "M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.419 1.56-.299.421-1.02.599-1.559.3z");
				add_location(path, file$3, 440, 26, 15463);
				attr_dev(svg, "class", "w-5 h-5 text-white");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "fill", "currentColor");
				add_location(svg, file$3, 439, 24, 15364);
				attr_dev(div, "class", "w-8 h-8 rounded-full bg-green-500 flex items-center justify-center");
				add_location(div, file$3, 438, 22, 15259);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, svg);
				append_dev(svg, path);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_1$3.name,
			type: "if",
			source: "(393:20) {#if service.provider === 'spotify'}",
			ctx
		});

		return block;
	}

	// (410:18) {:else}
	function create_else_block$3(ctx) {
		let button;
		let mounted;
		let dispose;

		function click_handler_3() {
			return /*click_handler_3*/ ctx[19](/*service*/ ctx[25]);
		}

		const block = {
			c: function create() {
				button = element("button");
				button.textContent = "Connect";
				attr_dev(button, "class", "text-xs text-blue-400 hover:text-blue-300");
				add_location(button, file$3, 455, 20, 18333);
			},
			m: function mount(target, anchor) {
				insert_dev(target, button, anchor);

				if (!mounted) {
					dispose = listen_dev(button, "click", click_handler_3, false, false, false, false);
					mounted = true;
				}
			},
			p: function update(new_ctx, dirty) {
				ctx = new_ctx;
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(button);
				}

				mounted = false;
				dispose();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block$3.name,
			type: "else",
			source: "(410:18) {:else}",
			ctx
		});

		return block;
	}

	// (408:18) {#if service.connected}
	function create_if_block$3(ctx) {
		let span;

		const block = {
			c: function create() {
				span = element("span");
				span.textContent = "Connected";
				attr_dev(span, "class", "text-xs text-green-400");
				add_location(span, file$3, 453, 20, 18233);
			},
			m: function mount(target, anchor) {
				insert_dev(target, span, anchor);
			},
			p: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(span);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block$3.name,
			type: "if",
			source: "(408:18) {#if service.connected}",
			ctx
		});

		return block;
	}

	// (389:12) {#each connectedServices as service}
	function create_each_block(ctx) {
		let div2;
		let div1;
		let div0;
		let t0;
		let span;
		let t1_value = /*service*/ ctx[25].provider.replace('_', ' ') + "";
		let t1;
		let t2;
		let t3;

		function select_block_type_4(ctx, dirty) {
			if (/*service*/ ctx[25].provider === 'spotify') return create_if_block_1$3;
			if (/*service*/ ctx[25].provider === 'apple_music') return create_if_block_2$2;
		}

		let current_block_type = select_block_type_4(ctx);
		let if_block0 = current_block_type && current_block_type(ctx);

		function select_block_type_5(ctx, dirty) {
			if (/*service*/ ctx[25].connected) return create_if_block$3;
			return create_else_block$3;
		}

		let current_block_type_1 = select_block_type_5(ctx);
		let if_block1 = current_block_type_1(ctx);

		const block = {
			c: function create() {
				div2 = element("div");
				div1 = element("div");
				div0 = element("div");
				if (if_block0) if_block0.c();
				t0 = space();
				span = element("span");
				t1 = text(t1_value);
				t2 = space();
				if_block1.c();
				t3 = space();
				attr_dev(span, "class", "font-medium text-sm capitalize");
				add_location(span, file$3, 450, 20, 18057);
				attr_dev(div0, "class", "flex items-center space-x-3");
				add_location(div0, file$3, 436, 18, 15138);
				attr_dev(div1, "class", "flex items-center justify-between");
				add_location(div1, file$3, 435, 16, 15072);
				attr_dev(div2, "class", "bg-gray-800 rounded-lg p-3 border border-gray-600");
				add_location(div2, file$3, 434, 14, 14992);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div2, anchor);
				append_dev(div2, div1);
				append_dev(div1, div0);
				if (if_block0) if_block0.m(div0, null);
				append_dev(div0, t0);
				append_dev(div0, span);
				append_dev(span, t1);
				append_dev(div1, t2);
				if_block1.m(div1, null);
				append_dev(div2, t3);
			},
			p: function update(ctx, dirty) {
				if (current_block_type !== (current_block_type = select_block_type_4(ctx))) {
					if (if_block0) if_block0.d(1);
					if_block0 = current_block_type && current_block_type(ctx);

					if (if_block0) {
						if_block0.c();
						if_block0.m(div0, t0);
					}
				}

				if (dirty[0] & /*connectedServices*/ 128 && t1_value !== (t1_value = /*service*/ ctx[25].provider.replace('_', ' ') + "")) set_data_dev(t1, t1_value);

				if (current_block_type_1 === (current_block_type_1 = select_block_type_5(ctx)) && if_block1) {
					if_block1.p(ctx, dirty);
				} else {
					if_block1.d(1);
					if_block1 = current_block_type_1(ctx);

					if (if_block1) {
						if_block1.c();
						if_block1.m(div1, null);
					}
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div2);
				}

				if (if_block0) {
					if_block0.d();
				}

				if_block1.d();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_each_block.name,
			type: "each",
			source: "(389:12) {#each connectedServices as service}",
			ctx
		});

		return block;
	}

	function create_fragment$3(ctx) {
		let div7;
		let header;
		let div2;
		let div1;
		let div0;
		let span0;
		let t1;
		let h1;
		let t3;
		let button;
		let svg;
		let path0;
		let path1;
		let t4;
		let main;
		let section0;
		let div3;
		let input;
		let t5;
		let t6;
		let t7;
		let div6;
		let section1;
		let h20;
		let span1;
		let t8;
		let t9;
		let t10;
		let aside;
		let section2;
		let h21;
		let t12;
		let t13;
		let section3;
		let h22;
		let t15;
		let div4;
		let t16;
		let section4;
		let h3;
		let t18;
		let div5;
		let t19_value = /*categoryLists*/ ctx[5].filter(func).reduce(func_1, 0) + "";
		let t19;
		let t20;
		let p;
		let mounted;
		let dispose;
		let if_block0 = /*isSearching*/ ctx[2] && create_if_block_11(ctx);
		let if_block1 = /*searchResults*/ ctx[1].length > 0 && create_if_block_6$1(ctx);

		function select_block_type_2(ctx, dirty) {
			if (/*isLoadingNews*/ ctx[4]) return create_if_block_5$1;
			return create_else_block_2$1;
		}

		let current_block_type = select_block_type_2(ctx);
		let if_block2 = current_block_type(ctx);

		function select_block_type_3(ctx, dirty) {
			if (/*isLoadingCategories*/ ctx[6]) return create_if_block_3$2;
			return create_else_block_1$2;
		}

		let current_block_type_1 = select_block_type_3(ctx);
		let if_block3 = current_block_type_1(ctx);
		let each_value = ensure_array_like_dev(/*connectedServices*/ ctx[7]);
		let each_blocks = [];

		for (let i = 0; i < each_value.length; i += 1) {
			each_blocks[i] = create_each_block(get_each_context(ctx, each_value, i));
		}

		const block = {
			c: function create() {
				div7 = element("div");
				header = element("header");
				div2 = element("div");
				div1 = element("div");
				div0 = element("div");
				span0 = element("span");
				span0.textContent = "🚫";
				t1 = space();
				h1 = element("h1");
				h1.textContent = "No Drake";
				t3 = space();
				button = element("button");
				svg = svg_element("svg");
				path0 = svg_element("path");
				path1 = svg_element("path");
				t4 = space();
				main = element("main");
				section0 = element("section");
				div3 = element("div");
				input = element("input");
				t5 = space();
				if (if_block0) if_block0.c();
				t6 = space();
				if (if_block1) if_block1.c();
				t7 = space();
				div6 = element("div");
				section1 = element("section");
				h20 = element("h2");
				span1 = element("span");
				t8 = text("\n          Recent Additions");
				t9 = space();
				if_block2.c();
				t10 = space();
				aside = element("aside");
				section2 = element("section");
				h21 = element("h2");
				h21.textContent = "Blocklists";
				t12 = space();
				if_block3.c();
				t13 = space();
				section3 = element("section");
				h22 = element("h2");
				h22.textContent = "Music Services";
				t15 = space();
				div4 = element("div");

				for (let i = 0; i < each_blocks.length; i += 1) {
					each_blocks[i].c();
				}

				t16 = space();
				section4 = element("section");
				h3 = element("h3");
				h3.textContent = "Your Blocks";
				t18 = space();
				div5 = element("div");
				t19 = text(t19_value);
				t20 = space();
				p = element("p");
				p.textContent = "artists blocked";
				attr_dev(span0, "class", "text-3xl");
				add_location(span0, file$3, 259, 10, 6719);
				attr_dev(h1, "class", "text-xl font-bold");
				add_location(h1, file$3, 260, 10, 6762);
				attr_dev(div0, "class", "flex items-center space-x-3");
				add_location(div0, file$3, 258, 8, 6667);
				attr_dev(path0, "stroke-linecap", "round");
				attr_dev(path0, "stroke-linejoin", "round");
				attr_dev(path0, "stroke-width", "2");
				attr_dev(path0, "d", "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z");
				add_location(path0, file$3, 267, 12, 7076);
				attr_dev(path1, "stroke-linecap", "round");
				attr_dev(path1, "stroke-linejoin", "round");
				attr_dev(path1, "stroke-width", "2");
				attr_dev(path1, "d", "M15 12a3 3 0 11-6 0 3 3 0 016 0z");
				add_location(path1, file$3, 268, 12, 7649);
				attr_dev(svg, "class", "w-6 h-6");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "stroke", "currentColor");
				attr_dev(svg, "viewBox", "0 0 24 24");
				add_location(svg, file$3, 266, 10, 6988);
				attr_dev(button, "class", "p-2 rounded-lg hover:bg-gray-700 transition-colors");
				add_location(button, file$3, 262, 8, 6829);
				attr_dev(div1, "class", "flex items-center justify-between");
				add_location(div1, file$3, 257, 6, 6611);
				attr_dev(div2, "class", "max-w-6xl mx-auto px-4 py-4");
				add_location(div2, file$3, 256, 4, 6563);
				attr_dev(header, "class", "bg-gray-800 border-b border-gray-600 sticky top-0 z-50");
				add_location(header, file$3, 255, 2, 6487);
				attr_dev(input, "type", "text");
				attr_dev(input, "placeholder", "Search artist to block...");
				attr_dev(input, "class", "w-full bg-gray-800 border border-gray-600 rounded-xl px-5 py-4 text-lg placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-rose-500 focus:border-transparent");
				add_location(input, file$3, 279, 8, 7965);
				attr_dev(div3, "class", "relative");
				add_location(div3, file$3, 278, 6, 7934);
				add_location(section0, file$3, 277, 4, 7918);
				attr_dev(span1, "class", "w-2 h-2 bg-red-500 rounded-full mr-2 animate-pulse");
				add_location(span1, file$3, 341, 10, 10751);
				attr_dev(h20, "class", "text-lg font-semibold mb-4 flex items-center");
				add_location(h20, file$3, 340, 8, 10683);
				attr_dev(section1, "class", "lg:col-span-2");
				add_location(section1, file$3, 339, 6, 10643);
				attr_dev(h21, "class", "text-lg font-semibold mb-4");
				add_location(h21, file$3, 386, 10, 12744);
				add_location(section2, file$3, 385, 8, 12724);
				attr_dev(h22, "class", "text-lg font-semibold mb-4");
				add_location(h22, file$3, 431, 10, 14836);
				attr_dev(div4, "class", "space-y-2");
				add_location(div4, file$3, 432, 10, 14905);
				add_location(section3, file$3, 430, 8, 14816);
				attr_dev(h3, "class", "text-sm font-medium text-gray-300 mb-3");
				add_location(h3, file$3, 470, 10, 18801);
				attr_dev(div5, "class", "text-3xl font-bold");
				add_location(div5, file$3, 471, 10, 18879);
				attr_dev(p, "class", "text-sm text-gray-400 mt-1");
				add_location(p, file$3, 474, 10, 19037);
				attr_dev(section4, "class", "bg-gray-800 rounded-xl p-4 border border-gray-600");
				add_location(section4, file$3, 469, 8, 18723);
				attr_dev(aside, "class", "space-y-6");
				add_location(aside, file$3, 383, 6, 12653);
				attr_dev(div6, "class", "grid lg:grid-cols-3 gap-6");
				add_location(div6, file$3, 337, 4, 10572);
				attr_dev(main, "class", "max-w-6xl mx-auto px-4 py-6 space-y-8");
				add_location(main, file$3, 275, 2, 7833);
				attr_dev(div7, "class", "min-h-screen bg-gray-900 text-white");
				add_location(div7, file$3, 253, 0, 6417);
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				insert_dev(target, div7, anchor);
				append_dev(div7, header);
				append_dev(header, div2);
				append_dev(div2, div1);
				append_dev(div1, div0);
				append_dev(div0, span0);
				append_dev(div0, t1);
				append_dev(div0, h1);
				append_dev(div1, t3);
				append_dev(div1, button);
				append_dev(button, svg);
				append_dev(svg, path0);
				append_dev(svg, path1);
				append_dev(div7, t4);
				append_dev(div7, main);
				append_dev(main, section0);
				append_dev(section0, div3);
				append_dev(div3, input);
				set_input_value(input, /*searchQuery*/ ctx[0]);
				append_dev(div3, t5);
				if (if_block0) if_block0.m(div3, null);
				append_dev(section0, t6);
				if (if_block1) if_block1.m(section0, null);
				append_dev(main, t7);
				append_dev(main, div6);
				append_dev(div6, section1);
				append_dev(section1, h20);
				append_dev(h20, span1);
				append_dev(h20, t8);
				append_dev(section1, t9);
				if_block2.m(section1, null);
				append_dev(div6, t10);
				append_dev(div6, aside);
				append_dev(aside, section2);
				append_dev(section2, h21);
				append_dev(section2, t12);
				if_block3.m(section2, null);
				append_dev(aside, t13);
				append_dev(aside, section3);
				append_dev(section3, h22);
				append_dev(section3, t15);
				append_dev(section3, div4);

				for (let i = 0; i < each_blocks.length; i += 1) {
					if (each_blocks[i]) {
						each_blocks[i].m(div4, null);
					}
				}

				append_dev(aside, t16);
				append_dev(aside, section4);
				append_dev(section4, h3);
				append_dev(section4, t18);
				append_dev(section4, div5);
				append_dev(div5, t19);
				append_dev(section4, t20);
				append_dev(section4, p);

				if (!mounted) {
					dispose = [
						listen_dev(button, "click", /*click_handler*/ ctx[15], false, false, false, false),
						listen_dev(input, "input", /*input_input_handler*/ ctx[16]),
						listen_dev(input, "input", /*handleSearchInput*/ ctx[11], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (dirty[0] & /*searchQuery*/ 1 && input.value !== /*searchQuery*/ ctx[0]) {
					set_input_value(input, /*searchQuery*/ ctx[0]);
				}

				if (/*isSearching*/ ctx[2]) {
					if (if_block0) ; else {
						if_block0 = create_if_block_11(ctx);
						if_block0.c();
						if_block0.m(div3, null);
					}
				} else if (if_block0) {
					if_block0.d(1);
					if_block0 = null;
				}

				if (/*searchResults*/ ctx[1].length > 0) {
					if (if_block1) {
						if_block1.p(ctx, dirty);
					} else {
						if_block1 = create_if_block_6$1(ctx);
						if_block1.c();
						if_block1.m(section0, null);
					}
				} else if (if_block1) {
					if_block1.d(1);
					if_block1 = null;
				}

				if (current_block_type === (current_block_type = select_block_type_2(ctx)) && if_block2) {
					if_block2.p(ctx, dirty);
				} else {
					if_block2.d(1);
					if_block2 = current_block_type(ctx);

					if (if_block2) {
						if_block2.c();
						if_block2.m(section1, null);
					}
				}

				if (current_block_type_1 === (current_block_type_1 = select_block_type_3(ctx)) && if_block3) {
					if_block3.p(ctx, dirty);
				} else {
					if_block3.d(1);
					if_block3 = current_block_type_1(ctx);

					if (if_block3) {
						if_block3.c();
						if_block3.m(section2, null);
					}
				}

				if (dirty[0] & /*connectedServices*/ 128) {
					each_value = ensure_array_like_dev(/*connectedServices*/ ctx[7]);
					let i;

					for (i = 0; i < each_value.length; i += 1) {
						const child_ctx = get_each_context(ctx, each_value, i);

						if (each_blocks[i]) {
							each_blocks[i].p(child_ctx, dirty);
						} else {
							each_blocks[i] = create_each_block(child_ctx);
							each_blocks[i].c();
							each_blocks[i].m(div4, null);
						}
					}

					for (; i < each_blocks.length; i += 1) {
						each_blocks[i].d(1);
					}

					each_blocks.length = each_value.length;
				}

				if (dirty[0] & /*categoryLists*/ 32 && t19_value !== (t19_value = /*categoryLists*/ ctx[5].filter(func).reduce(func_1, 0) + "")) set_data_dev(t19, t19_value);
			},
			i: noop,
			o: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div7);
				}

				if (if_block0) if_block0.d();
				if (if_block1) if_block1.d();
				if_block2.d();
				if_block3.d();
				destroy_each(each_blocks, detaching);
				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_fragment$3.name,
			type: "component",
			source: "",
			ctx
		});

		return block;
	}

	function formatTimeAgo(timestamp) {
		const seconds = Math.floor((Date.now() - new Date(timestamp).getTime()) / 1000);
		if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
		if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
		return `${Math.floor(seconds / 86400)}d ago`;
	}

	function connectService(provider) {
		// Will trigger OAuth flow
		window.location.href = `/api/v1/auth/oauth/${provider}/initiate`;
	}

	const func = c => c.subscribed;
	const func_1 = (sum, c) => sum + c.artist_count;

	function instance$3($$self, $$props, $$invalidate) {
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('Home', slots, []);
		let searchQuery = '';
		let searchResults = [];
		let isSearching = false;
		let searchTimeout;
		let newsFeed = [];
		let isLoadingNews = true;
		let categoryLists = [];
		let isLoadingCategories = true;
		let connectedServices = [];
		let blockingArtistId = null;

		// Category colors - improved contrast and reduced red overuse
		const categoryColors = {
			'sexual_misconduct': 'bg-rose-600',
			'sexual_assault': 'bg-rose-700',
			'domestic_violence': 'bg-red-600',
			'child_abuse': 'bg-red-800',
			'violent_crime': 'bg-red-500',
			'drug_trafficking': 'bg-purple-600',
			'hate_speech': 'bg-orange-600',
			'racism': 'bg-orange-700',
			'homophobia': 'bg-amber-600',
			'antisemitism': 'bg-amber-700',
			'fraud': 'bg-blue-600',
			'animal_abuse': 'bg-emerald-600',
			'other': 'bg-slate-500'
		};

		const categoryLabels = {
			'sexual_misconduct': 'Sexual Misconduct',
			'violence': 'Violence',
			'domestic_abuse': 'Domestic Abuse',
			'drug_trafficking': 'Drug Trafficking',
			'hate_speech': 'Hate Speech',
			'fraud': 'Fraud',
			'child_abuse': 'Child Abuse',
			'other': 'Other'
		};

		onMount(async () => {
			await Promise.all([loadNewsFeed(), loadCategories(), loadConnectedServices()]);
		});

		async function loadNewsFeed() {
			$$invalidate(4, isLoadingNews = true);

			try {
				// Mock data for now - will be replaced with AI-curated news API
				$$invalidate(3, newsFeed = [
					{
						id: '1',
						artist_name: 'Example Artist',
						category: 'violence',
						headline: 'Artist convicted on federal charges',
						source: 'AP News',
						source_url: 'https://apnews.com',
						timestamp: new Date(Date.now() - 3600000).toISOString()
					},
					{
						id: '2',
						artist_name: 'Another Artist',
						category: 'domestic_abuse',
						headline: 'Multiple victims come forward with allegations',
						source: 'Rolling Stone',
						source_url: 'https://rollingstone.com',
						timestamp: new Date(Date.now() - 86400000).toISOString()
					},
					{
						id: '3',
						artist_name: 'Third Artist',
						category: 'fraud',
						headline: 'Charged with wire fraud and money laundering',
						source: 'Billboard',
						source_url: 'https://billboard.com',
						timestamp: new Date(Date.now() - 172800000).toISOString()
					}
				]);
			} catch(e) {
				console.error('Failed to load news feed:', e);
			} finally {
				$$invalidate(4, isLoadingNews = false);
			}
		}

		async function loadCategories() {
			$$invalidate(6, isLoadingCategories = true);

			try {
				const result = await apiClient.get('/api/v1/categories');

				if (result.success && result.data) {
					$$invalidate(5, categoryLists = result.data.map(cat => ({
						...cat,
						color: categoryColors[cat.id] || 'bg-gray-500'
					})));
				}
			} catch(e) {
				console.error('Failed to load categories:', e);
			} finally {
				$$invalidate(6, isLoadingCategories = false);
			}
		}

		async function loadConnectedServices() {
			try {
				$$invalidate(7, connectedServices = [
					{ provider: 'spotify', connected: false },
					{
						provider: 'apple_music',
						connected: false
					}
				]);
			} catch(e) {
				console.error(
					'Failed to load services:',
					e
				);
			}
		}

		function handleSearchInput() {
			clearTimeout(searchTimeout);

			if (searchQuery.length < 2) {
				$$invalidate(1, searchResults = []);
				return;
			}

			searchTimeout = setTimeout(() => searchArtists(), 300);
		}

		async function searchArtists() {
			if (searchQuery.length < 2) return;
			$$invalidate(2, isSearching = true);

			try {
				const result = await apiClient.get(`/api/v1/dnp/search?q=${encodeURIComponent(searchQuery)}`);

				if (result.success && result.data) {
					$$invalidate(1, searchResults = result.data);
				}
			} catch(e) {
				console.error('Search failed:', e);
			} finally {
				$$invalidate(2, isSearching = false);
			}
		}

		async function blockArtist(artist) {
			$$invalidate(8, blockingArtistId = artist.id);

			try {
				const result = await apiClient.post('/api/v1/dnp/list', {
					artist_id: artist.id,
					reason: 'User blocked'
				});

				if (result.success) {
					// Update local state
					$$invalidate(1, searchResults = searchResults.map(a => a.id === artist.id ? { ...a, blocked: true } : a));
				}
			} catch(e) {
				console.error('Failed to block artist:', e);
			} finally {
				$$invalidate(8, blockingArtistId = null);
			}
		}

		async function unblockArtist(artist) {
			$$invalidate(8, blockingArtistId = artist.id);

			try {
				const result = await apiClient.delete(`/api/v1/dnp/list/${artist.id}`);

				if (result.success) {
					$$invalidate(1, searchResults = searchResults.map(a => a.id === artist.id ? { ...a, blocked: false } : a));
				}
			} catch(e) {
				console.error('Failed to unblock artist:', e);
			} finally {
				$$invalidate(8, blockingArtistId = null);
			}
		}

		async function toggleCategory(category) {
			const wasSubscribed = category.subscribed;

			// Optimistic update
			$$invalidate(5, categoryLists = categoryLists.map(c => c.id === category.id
			? { ...c, subscribed: !wasSubscribed }
			: c));

			try {
				if (wasSubscribed) {
					await apiClient.delete(`/api/v1/categories/${category.id}/subscribe`);
				} else {
					await apiClient.post(`/api/v1/categories/${category.id}/subscribe`);
				}
			} catch(e) {
				// Revert on error
				$$invalidate(5, categoryLists = categoryLists.map(c => c.id === category.id
				? { ...c, subscribed: wasSubscribed }
				: c));

				console.error('Failed to toggle category:', e);
			}
		}

		const writable_props = [];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console_1$2.warn(`<Home> was created with unknown prop '${key}'`);
		});

		const click_handler = () => window.location.href = '/settings';

		function input_input_handler() {
			searchQuery = this.value;
			$$invalidate(0, searchQuery);
		}

		const click_handler_1 = artist => artist.blocked
		? unblockArtist(artist)
		: blockArtist(artist);

		const click_handler_2 = category => toggleCategory(category);
		const click_handler_3 = service => connectService(service.provider);

		$$self.$capture_state = () => ({
			onMount,
			apiClient,
			searchQuery,
			searchResults,
			isSearching,
			searchTimeout,
			newsFeed,
			isLoadingNews,
			categoryLists,
			isLoadingCategories,
			connectedServices,
			blockingArtistId,
			categoryColors,
			categoryLabels,
			loadNewsFeed,
			loadCategories,
			loadConnectedServices,
			handleSearchInput,
			searchArtists,
			blockArtist,
			unblockArtist,
			toggleCategory,
			formatTimeAgo,
			connectService
		});

		$$self.$inject_state = $$props => {
			if ('searchQuery' in $$props) $$invalidate(0, searchQuery = $$props.searchQuery);
			if ('searchResults' in $$props) $$invalidate(1, searchResults = $$props.searchResults);
			if ('isSearching' in $$props) $$invalidate(2, isSearching = $$props.isSearching);
			if ('searchTimeout' in $$props) searchTimeout = $$props.searchTimeout;
			if ('newsFeed' in $$props) $$invalidate(3, newsFeed = $$props.newsFeed);
			if ('isLoadingNews' in $$props) $$invalidate(4, isLoadingNews = $$props.isLoadingNews);
			if ('categoryLists' in $$props) $$invalidate(5, categoryLists = $$props.categoryLists);
			if ('isLoadingCategories' in $$props) $$invalidate(6, isLoadingCategories = $$props.isLoadingCategories);
			if ('connectedServices' in $$props) $$invalidate(7, connectedServices = $$props.connectedServices);
			if ('blockingArtistId' in $$props) $$invalidate(8, blockingArtistId = $$props.blockingArtistId);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		return [
			searchQuery,
			searchResults,
			isSearching,
			newsFeed,
			isLoadingNews,
			categoryLists,
			isLoadingCategories,
			connectedServices,
			blockingArtistId,
			categoryColors,
			categoryLabels,
			handleSearchInput,
			blockArtist,
			unblockArtist,
			toggleCategory,
			click_handler,
			input_input_handler,
			click_handler_1,
			click_handler_2,
			click_handler_3
		];
	}

	class Home extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance$3, create_fragment$3, safe_not_equal, {}, null, [-1, -1]);

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "Home",
				options,
				id: create_fragment$3.name
			});
		}
	}

	/* src/lib/components/Settings.svelte generated by Svelte v4.2.20 */

	const { console: console_1$1 } = globals;
	const file$2 = "src/lib/components/Settings.svelte";

	// (121:6) {#if connectionError}
	function create_if_block_7(ctx) {
		let div;
		let t;

		const block = {
			c: function create() {
				div = element("div");
				t = text(/*connectionError*/ ctx[3]);
				attr_dev(div, "class", "mx-4 mt-4 p-3 bg-red-900/30 border border-red-700 rounded-lg text-sm text-red-300");
				add_location(div, file$2, 130, 8, 3856);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, t);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*connectionError*/ 8) set_data_dev(t, /*connectionError*/ ctx[3]);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_7.name,
			type: "if",
			source: "(121:6) {#if connectionError}",
			ctx
		});

		return block;
	}

	// (142:14) {:else}
	function create_else_block_4(ctx) {
		let p;

		const block = {
			c: function create() {
				p = element("p");
				p.textContent = "Not connected";
				attr_dev(p, "class", "text-sm text-gray-300");
				add_location(p, file$2, 151, 16, 5349);
			},
			m: function mount(target, anchor) {
				insert_dev(target, p, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(p);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block_4.name,
			type: "else",
			source: "(142:14) {:else}",
			ctx
		});

		return block;
	}

	// (140:47) 
	function create_if_block_6(ctx) {
		let p;

		const block = {
			c: function create() {
				p = element("p");
				p.textContent = "Connected";
				attr_dev(p, "class", "text-sm text-green-400");
				add_location(p, file$2, 149, 16, 5263);
			},
			m: function mount(target, anchor) {
				insert_dev(target, p, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(p);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_6.name,
			type: "if",
			source: "(140:47) ",
			ctx
		});

		return block;
	}

	// (138:14) {#if isLoadingConnections}
	function create_if_block_5(ctx) {
		let p;

		const block = {
			c: function create() {
				p = element("p");
				p.textContent = "Loading...";
				attr_dev(p, "class", "text-sm text-gray-400");
				add_location(p, file$2, 147, 16, 5151);
			},
			m: function mount(target, anchor) {
				insert_dev(target, p, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(p);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_5.name,
			type: "if",
			source: "(138:14) {#if isLoadingConnections}",
			ctx
		});

		return block;
	}

	// (155:10) {:else}
	function create_else_block_3(ctx) {
		let button;

		let t_value = (/*connectingProvider*/ ctx[2] === 'spotify'
		? 'Connecting...'
		: 'Connect') + "";

		let t;
		let button_disabled_value;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				button = element("button");
				t = text(t_value);
				button.disabled = button_disabled_value = /*connectingProvider*/ ctx[2] === 'spotify';
				attr_dev(button, "class", "px-4 py-2 bg-green-600 hover:bg-green-700 rounded-lg text-sm font-medium transition-colors disabled:opacity-50");
				add_location(button, file$2, 164, 12, 5915);
			},
			m: function mount(target, anchor) {
				insert_dev(target, button, anchor);
				append_dev(button, t);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*click_handler_2*/ ctx[11], false, false, false, false);
					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (dirty & /*connectingProvider*/ 4 && t_value !== (t_value = (/*connectingProvider*/ ctx[2] === 'spotify'
				? 'Connecting...'
				: 'Connect') + "")) set_data_dev(t, t_value);

				if (dirty & /*connectingProvider*/ 4 && button_disabled_value !== (button_disabled_value = /*connectingProvider*/ ctx[2] === 'spotify')) {
					prop_dev(button, "disabled", button_disabled_value);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(button);
				}

				mounted = false;
				dispose();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block_3.name,
			type: "else",
			source: "(155:10) {:else}",
			ctx
		});

		return block;
	}

	// (147:10) {#if isConnected('spotify')}
	function create_if_block_4$1(ctx) {
		let button;

		let t_value = (/*connectingProvider*/ ctx[2] === 'spotify'
		? 'Disconnecting...'
		: 'Disconnect') + "";

		let t;
		let button_disabled_value;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				button = element("button");
				t = text(t_value);
				button.disabled = button_disabled_value = /*connectingProvider*/ ctx[2] === 'spotify';
				attr_dev(button, "class", "px-4 py-2 bg-gray-600 hover:bg-gray-500 rounded-lg text-sm font-medium transition-colors disabled:opacity-50");
				add_location(button, file$2, 156, 12, 5507);
			},
			m: function mount(target, anchor) {
				insert_dev(target, button, anchor);
				append_dev(button, t);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*click_handler_1*/ ctx[10], false, false, false, false);
					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (dirty & /*connectingProvider*/ 4 && t_value !== (t_value = (/*connectingProvider*/ ctx[2] === 'spotify'
				? 'Disconnecting...'
				: 'Disconnect') + "")) set_data_dev(t, t_value);

				if (dirty & /*connectingProvider*/ 4 && button_disabled_value !== (button_disabled_value = /*connectingProvider*/ ctx[2] === 'spotify')) {
					prop_dev(button, "disabled", button_disabled_value);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(button);
				}

				mounted = false;
				dispose();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_4$1.name,
			type: "if",
			source: "(147:10) {#if isConnected('spotify')}",
			ctx
		});

		return block;
	}

	// (180:14) {:else}
	function create_else_block_2(ctx) {
		let p;

		const block = {
			c: function create() {
				p = element("p");
				p.textContent = "Not connected";
				attr_dev(p, "class", "text-sm text-gray-300");
				add_location(p, file$2, 189, 16, 8557);
			},
			m: function mount(target, anchor) {
				insert_dev(target, p, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(p);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block_2.name,
			type: "else",
			source: "(180:14) {:else}",
			ctx
		});

		return block;
	}

	// (178:45) 
	function create_if_block_3$1(ctx) {
		let p;

		const block = {
			c: function create() {
				p = element("p");
				p.textContent = "Connected";
				attr_dev(p, "class", "text-sm text-green-400");
				add_location(p, file$2, 187, 16, 8471);
			},
			m: function mount(target, anchor) {
				insert_dev(target, p, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(p);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_3$1.name,
			type: "if",
			source: "(178:45) ",
			ctx
		});

		return block;
	}

	// (176:14) {#if isLoadingConnections}
	function create_if_block_2$1(ctx) {
		let p;

		const block = {
			c: function create() {
				p = element("p");
				p.textContent = "Loading...";
				attr_dev(p, "class", "text-sm text-gray-400");
				add_location(p, file$2, 185, 16, 8361);
			},
			m: function mount(target, anchor) {
				insert_dev(target, p, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(p);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_2$1.name,
			type: "if",
			source: "(176:14) {#if isLoadingConnections}",
			ctx
		});

		return block;
	}

	// (193:10) {:else}
	function create_else_block_1$1(ctx) {
		let button;

		let t_value = (/*connectingProvider*/ ctx[2] === 'apple'
		? 'Connecting...'
		: 'Connect') + "";

		let t;
		let button_disabled_value;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				button = element("button");
				t = text(t_value);
				button.disabled = button_disabled_value = /*connectingProvider*/ ctx[2] === 'apple';
				attr_dev(button, "class", "px-4 py-2 bg-pink-600 hover:bg-pink-700 rounded-lg text-sm font-medium transition-colors disabled:opacity-50");
				add_location(button, file$2, 202, 12, 9115);
			},
			m: function mount(target, anchor) {
				insert_dev(target, button, anchor);
				append_dev(button, t);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*click_handler_4*/ ctx[13], false, false, false, false);
					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (dirty & /*connectingProvider*/ 4 && t_value !== (t_value = (/*connectingProvider*/ ctx[2] === 'apple'
				? 'Connecting...'
				: 'Connect') + "")) set_data_dev(t, t_value);

				if (dirty & /*connectingProvider*/ 4 && button_disabled_value !== (button_disabled_value = /*connectingProvider*/ ctx[2] === 'apple')) {
					prop_dev(button, "disabled", button_disabled_value);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(button);
				}

				mounted = false;
				dispose();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block_1$1.name,
			type: "else",
			source: "(193:10) {:else}",
			ctx
		});

		return block;
	}

	// (185:10) {#if isConnected('apple')}
	function create_if_block_1$2(ctx) {
		let button;

		let t_value = (/*connectingProvider*/ ctx[2] === 'apple'
		? 'Disconnecting...'
		: 'Disconnect') + "";

		let t;
		let button_disabled_value;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				button = element("button");
				t = text(t_value);
				button.disabled = button_disabled_value = /*connectingProvider*/ ctx[2] === 'apple';
				attr_dev(button, "class", "px-4 py-2 bg-gray-600 hover:bg-gray-500 rounded-lg text-sm font-medium transition-colors disabled:opacity-50");
				add_location(button, file$2, 194, 12, 8713);
			},
			m: function mount(target, anchor) {
				insert_dev(target, button, anchor);
				append_dev(button, t);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*click_handler_3*/ ctx[12], false, false, false, false);
					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (dirty & /*connectingProvider*/ 4 && t_value !== (t_value = (/*connectingProvider*/ ctx[2] === 'apple'
				? 'Disconnecting...'
				: 'Disconnect') + "")) set_data_dev(t, t_value);

				if (dirty & /*connectingProvider*/ 4 && button_disabled_value !== (button_disabled_value = /*connectingProvider*/ ctx[2] === 'apple')) {
					prop_dev(button, "disabled", button_disabled_value);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(button);
				}

				mounted = false;
				dispose();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_1$2.name,
			type: "if",
			source: "(185:10) {#if isConnected('apple')}",
			ctx
		});

		return block;
	}

	// (259:10) {:else}
	function create_else_block$2(ctx) {
		let svg;
		let path;
		let t0;
		let span;

		const block = {
			c: function create() {
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				span = element("span");
				span.textContent = "Sign out";
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1");
				add_location(path, file$2, 269, 14, 12919);
				attr_dev(svg, "class", "w-5 h-5");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "stroke", "currentColor");
				attr_dev(svg, "viewBox", "0 0 24 24");
				add_location(svg, file$2, 268, 12, 12829);
				add_location(span, file$2, 271, 12, 13117);
			},
			m: function mount(target, anchor) {
				insert_dev(target, svg, anchor);
				append_dev(svg, path);
				insert_dev(target, t0, anchor);
				insert_dev(target, span, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(svg);
					detach_dev(t0);
					detach_dev(span);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block$2.name,
			type: "else",
			source: "(259:10) {:else}",
			ctx
		});

		return block;
	}

	// (256:10) {#if isLoggingOut}
	function create_if_block$2(ctx) {
		let div;
		let t0;
		let span;

		const block = {
			c: function create() {
				div = element("div");
				t0 = space();
				span = element("span");
				span.textContent = "Signing out...";
				attr_dev(div, "class", "w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin");
				add_location(div, file$2, 265, 12, 12662);
				add_location(span, file$2, 266, 12, 12771);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				insert_dev(target, t0, anchor);
				insert_dev(target, span, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
					detach_dev(t0);
					detach_dev(span);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block$2.name,
			type: "if",
			source: "(256:10) {#if isLoggingOut}",
			ctx
		});

		return block;
	}

	function create_fragment$2(ctx) {
		let div28;
		let header;
		let div1;
		let div0;
		let button0;
		let svg0;
		let path0;
		let t0;
		let h1;
		let t2;
		let main;
		let section0;
		let div2;
		let h20;
		let t4;
		let div4;
		let div3;
		let label0;
		let t6;
		let p0;
		let t7_value = (/*$currentUser*/ ctx[4]?.email || 'Not signed in') + "";
		let t7;
		let t8;
		let section1;
		let div5;
		let h21;
		let t10;
		let t11;
		let div14;
		let div9;
		let div8;
		let div6;
		let svg1;
		let path1;
		let t12;
		let div7;
		let p1;
		let t14;
		let show_if_3;
		let t15;
		let t16;
		let div13;
		let div12;
		let div10;
		let svg2;
		let path2;
		let t17;
		let div11;
		let p2;
		let t19;
		let show_if_1;
		let t20;
		let t21;
		let section2;
		let div15;
		let h22;
		let t23;
		let div25;
		let div18;
		let div16;
		let p3;
		let t25;
		let p4;
		let t27;
		let label1;
		let input0;
		let t28;
		let div17;
		let t29;
		let div21;
		let div19;
		let p5;
		let t31;
		let p6;
		let t33;
		let label2;
		let input1;
		let t34;
		let div20;
		let t35;
		let div24;
		let div22;
		let p7;
		let t37;
		let p8;
		let t39;
		let label3;
		let input2;
		let t40;
		let div23;
		let t41;
		let section3;
		let div26;
		let h23;
		let t43;
		let div27;
		let button1;
		let t44;
		let button2;
		let t46;
		let p9;
		let mounted;
		let dispose;
		let if_block0 = /*connectionError*/ ctx[3] && create_if_block_7(ctx);

		function select_block_type(ctx, dirty) {
			if (/*isLoadingConnections*/ ctx[1]) return create_if_block_5;
			if (show_if_3 == null) show_if_3 = !!/*isConnected*/ ctx[5]('spotify');
			if (show_if_3) return create_if_block_6;
			return create_else_block_4;
		}

		let current_block_type = select_block_type(ctx);
		let if_block1 = current_block_type(ctx);

		function select_block_type_1(ctx, dirty) {
			if (/*isConnected*/ ctx[5]('spotify')) return create_if_block_4$1;
			return create_else_block_3;
		}

		let current_block_type_1 = select_block_type_1(ctx);
		let if_block2 = current_block_type_1(ctx);

		function select_block_type_2(ctx, dirty) {
			if (/*isLoadingConnections*/ ctx[1]) return create_if_block_2$1;
			if (show_if_1 == null) show_if_1 = !!/*isConnected*/ ctx[5]('apple');
			if (show_if_1) return create_if_block_3$1;
			return create_else_block_2;
		}

		let current_block_type_2 = select_block_type_2(ctx);
		let if_block3 = current_block_type_2(ctx);

		function select_block_type_3(ctx, dirty) {
			if (/*isConnected*/ ctx[5]('apple')) return create_if_block_1$2;
			return create_else_block_1$1;
		}

		let current_block_type_3 = select_block_type_3(ctx);
		let if_block4 = current_block_type_3(ctx);

		function select_block_type_4(ctx, dirty) {
			if (/*isLoggingOut*/ ctx[0]) return create_if_block$2;
			return create_else_block$2;
		}

		let current_block_type_4 = select_block_type_4(ctx);
		let if_block5 = current_block_type_4(ctx);

		const block = {
			c: function create() {
				div28 = element("div");
				header = element("header");
				div1 = element("div");
				div0 = element("div");
				button0 = element("button");
				svg0 = svg_element("svg");
				path0 = svg_element("path");
				t0 = space();
				h1 = element("h1");
				h1.textContent = "Settings";
				t2 = space();
				main = element("main");
				section0 = element("section");
				div2 = element("div");
				h20 = element("h2");
				h20.textContent = "Account";
				t4 = space();
				div4 = element("div");
				div3 = element("div");
				label0 = element("label");
				label0.textContent = "Email";
				t6 = space();
				p0 = element("p");
				t7 = text(t7_value);
				t8 = space();
				section1 = element("section");
				div5 = element("div");
				h21 = element("h2");
				h21.textContent = "Music Services";
				t10 = space();
				if (if_block0) if_block0.c();
				t11 = space();
				div14 = element("div");
				div9 = element("div");
				div8 = element("div");
				div6 = element("div");
				svg1 = svg_element("svg");
				path1 = svg_element("path");
				t12 = space();
				div7 = element("div");
				p1 = element("p");
				p1.textContent = "Spotify";
				t14 = space();
				if_block1.c();
				t15 = space();
				if_block2.c();
				t16 = space();
				div13 = element("div");
				div12 = element("div");
				div10 = element("div");
				svg2 = svg_element("svg");
				path2 = svg_element("path");
				t17 = space();
				div11 = element("div");
				p2 = element("p");
				p2.textContent = "Apple Music";
				t19 = space();
				if_block3.c();
				t20 = space();
				if_block4.c();
				t21 = space();
				section2 = element("section");
				div15 = element("div");
				h22 = element("h2");
				h22.textContent = "Preferences";
				t23 = space();
				div25 = element("div");
				div18 = element("div");
				div16 = element("div");
				p3 = element("p");
				p3.textContent = "Block featured artists";
				t25 = space();
				p4 = element("p");
				p4.textContent = "Also block songs where artist is featured";
				t27 = space();
				label1 = element("label");
				input0 = element("input");
				t28 = space();
				div17 = element("div");
				t29 = space();
				div21 = element("div");
				div19 = element("div");
				p5 = element("p");
				p5.textContent = "Block producer credits";
				t31 = space();
				p6 = element("p");
				p6.textContent = "Block songs produced by blocked artists";
				t33 = space();
				label2 = element("label");
				input1 = element("input");
				t34 = space();
				div20 = element("div");
				t35 = space();
				div24 = element("div");
				div22 = element("div");
				p7 = element("p");
				p7.textContent = "News notifications";
				t37 = space();
				p8 = element("p");
				p8.textContent = "Get notified when new artists are added";
				t39 = space();
				label3 = element("label");
				input2 = element("input");
				t40 = space();
				div23 = element("div");
				t41 = space();
				section3 = element("section");
				div26 = element("div");
				h23 = element("h2");
				h23.textContent = "Account Actions";
				t43 = space();
				div27 = element("div");
				button1 = element("button");
				if_block5.c();
				t44 = space();
				button2 = element("button");
				button2.textContent = "Delete account";
				t46 = space();
				p9 = element("p");
				p9.textContent = "No Drake v1.0.0";
				attr_dev(path0, "stroke-linecap", "round");
				attr_dev(path0, "stroke-linejoin", "round");
				attr_dev(path0, "stroke-width", "2");
				attr_dev(path0, "d", "M15 19l-7-7 7-7");
				add_location(path0, file$2, 101, 12, 2857);
				attr_dev(svg0, "class", "w-6 h-6");
				attr_dev(svg0, "fill", "none");
				attr_dev(svg0, "stroke", "currentColor");
				attr_dev(svg0, "viewBox", "0 0 24 24");
				add_location(svg0, file$2, 100, 10, 2769);
				attr_dev(button0, "class", "p-2 rounded-lg hover:bg-gray-700 transition-colors");
				add_location(button0, file$2, 96, 8, 2626);
				attr_dev(h1, "class", "text-xl font-bold");
				add_location(h1, file$2, 104, 8, 2993);
				attr_dev(div0, "class", "flex items-center space-x-4");
				add_location(div0, file$2, 95, 6, 2576);
				attr_dev(div1, "class", "max-w-2xl mx-auto px-4 py-4");
				add_location(div1, file$2, 94, 4, 2528);
				attr_dev(header, "class", "bg-gray-800 border-b border-gray-600 sticky top-0 z-50");
				add_location(header, file$2, 93, 2, 2452);
				attr_dev(h20, "class", "font-semibold");
				add_location(h20, file$2, 113, 8, 3305);
				attr_dev(div2, "class", "px-4 py-3 border-b border-gray-600");
				add_location(div2, file$2, 112, 6, 3248);
				attr_dev(label0, "class", "text-sm text-gray-300");
				add_location(label0, file$2, 117, 10, 3415);
				attr_dev(p0, "class", "mt-1");
				add_location(p0, file$2, 118, 10, 3476);
				add_location(div3, file$2, 116, 8, 3399);
				attr_dev(div4, "class", "p-4 space-y-4");
				add_location(div4, file$2, 115, 6, 3363);
				attr_dev(section0, "class", "bg-gray-800 rounded-xl border border-gray-600 overflow-hidden");
				add_location(section0, file$2, 111, 4, 3162);
				attr_dev(h21, "class", "font-semibold");
				add_location(h21, file$2, 126, 8, 3760);
				attr_dev(div5, "class", "px-4 py-3 border-b border-gray-600");
				add_location(div5, file$2, 125, 6, 3703);
				attr_dev(path1, "d", "M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.419 1.56-.299.421-1.02.599-1.559.3z");
				add_location(path1, file$2, 141, 16, 4388);
				attr_dev(svg1, "class", "w-6 h-6 text-white");
				attr_dev(svg1, "viewBox", "0 0 24 24");
				attr_dev(svg1, "fill", "currentColor");
				add_location(svg1, file$2, 140, 14, 4299);
				attr_dev(div6, "class", "w-10 h-10 rounded-full bg-green-500 flex items-center justify-center");
				add_location(div6, file$2, 139, 12, 4202);
				attr_dev(p1, "class", "font-medium");
				add_location(p1, file$2, 145, 14, 5059);
				add_location(div7, file$2, 144, 12, 5039);
				attr_dev(div8, "class", "flex items-center space-x-3");
				add_location(div8, file$2, 138, 10, 4148);
				attr_dev(div9, "class", "p-4 flex items-center justify-between");
				add_location(div9, file$2, 137, 8, 4086);
				attr_dev(path2, "d", "M23.994 6.124a9.23 9.23 0 00-.24-2.19c-.317-1.31-1.062-2.31-2.18-3.043a5.022 5.022 0 00-1.877-.726 10.496 10.496 0 00-1.564-.15c-.04-.003-.083-.01-.124-.013H5.986c-.152.01-.303.017-.455.026-.747.043-1.49.123-2.193.4-1.336.53-2.3 1.452-2.865 2.78-.192.448-.292.925-.363 1.408-.056.392-.088.785-.1 1.18 0 .032-.007.062-.01.093v12.223c.01.14.017.283.027.424.05.815.154 1.624.497 2.373.65 1.42 1.738 2.353 3.234 2.801.42.127.856.187 1.293.228.555.053 1.11.06 1.667.06h11.03a12.5 12.5 0 001.57-.1c.822-.106 1.596-.35 2.295-.81a5.046 5.046 0 001.88-2.207c.186-.42.293-.87.37-1.324.113-.675.138-1.358.137-2.04-.002-3.8 0-7.595-.003-11.393zm-6.423 3.99v5.712c0 .417-.058.827-.244 1.206-.29.59-.76.962-1.388 1.14-.35.1-.706.157-1.07.173-.95.042-1.785-.476-2.144-1.32-.238-.56-.223-1.136-.017-1.7.303-.825.96-1.277 1.743-1.49.294-.08.595-.13.893-.18.323-.054.65-.1.973-.157.274-.048.47-.202.53-.486a.707.707 0 00.017-.146c.002-1.633.002-3.265.002-4.898v-.07l-.06-.01c-2.097.4-4.194.8-6.29 1.202-.014.002-.032.014-.037.026-.006.016-.003.037-.003.056v7.36c0 .418-.052.832-.227 1.218-.282.622-.76 1.02-1.416 1.207-.313.09-.634.138-.96.166-.906.08-1.732-.4-2.134-1.203-.268-.534-.278-1.1-.096-1.66.267-.817.864-1.304 1.64-1.55.376-.12.763-.185 1.148-.25.278-.047.558-.088.832-.145.317-.065.522-.25.58-.574a.504.504 0 00.007-.115v-8.41c0-.25.042-.493.15-.72.183-.385.486-.62.882-.728.17-.047.346-.073.522-.11 2.55-.526 5.1-1.05 7.65-1.573.093-.02.19-.03.285-.03.316.004.528.2.613.5.032.113.044.233.044.35v5.9z");
				add_location(path2, file$2, 179, 16, 6686);
				attr_dev(svg2, "class", "w-6 h-6 text-white");
				attr_dev(svg2, "viewBox", "0 0 24 24");
				attr_dev(svg2, "fill", "currentColor");
				add_location(svg2, file$2, 178, 14, 6597);
				attr_dev(div10, "class", "w-10 h-10 rounded-full bg-gradient-to-br from-red-500 to-pink-500 flex items-center justify-center");
				add_location(div10, file$2, 177, 12, 6470);
				attr_dev(p2, "class", "font-medium");
				add_location(p2, file$2, 183, 14, 8265);
				add_location(div11, file$2, 182, 12, 8245);
				attr_dev(div12, "class", "flex items-center space-x-3");
				add_location(div12, file$2, 176, 10, 6416);
				attr_dev(div13, "class", "p-4 flex items-center justify-between");
				add_location(div13, file$2, 175, 8, 6354);
				attr_dev(div14, "class", "divide-y divide-gray-600");
				add_location(div14, file$2, 135, 6, 4014);
				attr_dev(section1, "class", "bg-gray-800 rounded-xl border border-gray-600 overflow-hidden");
				add_location(section1, file$2, 124, 4, 3617);
				attr_dev(h22, "class", "font-semibold");
				add_location(h22, file$2, 217, 8, 9709);
				attr_dev(div15, "class", "px-4 py-3 border-b border-gray-600");
				add_location(div15, file$2, 216, 6, 9652);
				attr_dev(p3, "class", "font-medium");
				add_location(p3, file$2, 222, 12, 9898);
				attr_dev(p4, "class", "text-sm text-gray-300");
				add_location(p4, file$2, 223, 12, 9960);
				add_location(div16, file$2, 221, 10, 9880);
				attr_dev(input0, "type", "checkbox");
				input0.checked = true;
				attr_dev(input0, "class", "sr-only peer");
				add_location(input0, file$2, 226, 12, 10143);
				attr_dev(div17, "class", "w-11 h-6 bg-gray-600 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-rose-600");
				add_location(div17, file$2, 227, 12, 10208);
				attr_dev(label1, "class", "relative inline-flex items-center cursor-pointer");
				add_location(label1, file$2, 225, 10, 10066);
				attr_dev(div18, "class", "p-4 flex items-center justify-between");
				add_location(div18, file$2, 220, 8, 9818);
				attr_dev(p5, "class", "font-medium");
				add_location(p5, file$2, 232, 12, 10648);
				attr_dev(p6, "class", "text-sm text-gray-300");
				add_location(p6, file$2, 233, 12, 10710);
				add_location(div19, file$2, 231, 10, 10630);
				attr_dev(input1, "type", "checkbox");
				attr_dev(input1, "class", "sr-only peer");
				add_location(input1, file$2, 236, 12, 10891);
				attr_dev(div20, "class", "w-11 h-6 bg-gray-600 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-rose-600");
				add_location(div20, file$2, 237, 12, 10948);
				attr_dev(label2, "class", "relative inline-flex items-center cursor-pointer");
				add_location(label2, file$2, 235, 10, 10814);
				attr_dev(div21, "class", "p-4 flex items-center justify-between");
				add_location(div21, file$2, 230, 8, 10568);
				attr_dev(p7, "class", "font-medium");
				add_location(p7, file$2, 242, 12, 11388);
				attr_dev(p8, "class", "text-sm text-gray-300");
				add_location(p8, file$2, 243, 12, 11446);
				add_location(div22, file$2, 241, 10, 11370);
				attr_dev(input2, "type", "checkbox");
				input2.checked = true;
				attr_dev(input2, "class", "sr-only peer");
				add_location(input2, file$2, 246, 12, 11627);
				attr_dev(div23, "class", "w-11 h-6 bg-gray-600 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-rose-600");
				add_location(div23, file$2, 247, 12, 11692);
				attr_dev(label3, "class", "relative inline-flex items-center cursor-pointer");
				add_location(label3, file$2, 245, 10, 11550);
				attr_dev(div24, "class", "p-4 flex items-center justify-between");
				add_location(div24, file$2, 240, 8, 11308);
				attr_dev(div25, "class", "divide-y divide-gray-600");
				add_location(div25, file$2, 219, 6, 9771);
				attr_dev(section2, "class", "bg-gray-800 rounded-xl border border-gray-600 overflow-hidden");
				add_location(section2, file$2, 215, 4, 9566);
				attr_dev(h23, "class", "font-semibold text-red-400");
				add_location(h23, file$2, 256, 8, 12243);
				attr_dev(div26, "class", "px-4 py-3 border-b border-red-900");
				add_location(div26, file$2, 255, 6, 12187);
				button1.disabled = /*isLoggingOut*/ ctx[0];
				attr_dev(button1, "class", "w-full px-4 py-3 bg-gray-700 hover:bg-gray-600 rounded-lg text-sm font-medium transition-colors flex items-center justify-center space-x-2 disabled:opacity-50");
				add_location(button1, file$2, 259, 8, 12358);
				attr_dev(button2, "class", "w-full px-4 py-3 bg-red-900/30 hover:bg-red-900/50 text-red-400 rounded-lg text-sm font-medium transition-colors");
				add_location(button2, file$2, 274, 8, 13181);
				attr_dev(div27, "class", "p-4 space-y-3");
				add_location(div27, file$2, 258, 6, 12322);
				attr_dev(section3, "class", "bg-gray-800 rounded-xl border border-red-900 overflow-hidden");
				add_location(section3, file$2, 254, 4, 12102);
				attr_dev(p9, "class", "text-center text-sm text-gray-400");
				add_location(p9, file$2, 281, 4, 13408);
				attr_dev(main, "class", "max-w-2xl mx-auto px-4 py-6 space-y-6");
				add_location(main, file$2, 109, 2, 3076);
				attr_dev(div28, "class", "min-h-screen bg-gray-900 text-white");
				add_location(div28, file$2, 91, 0, 2382);
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				insert_dev(target, div28, anchor);
				append_dev(div28, header);
				append_dev(header, div1);
				append_dev(div1, div0);
				append_dev(div0, button0);
				append_dev(button0, svg0);
				append_dev(svg0, path0);
				append_dev(div0, t0);
				append_dev(div0, h1);
				append_dev(div28, t2);
				append_dev(div28, main);
				append_dev(main, section0);
				append_dev(section0, div2);
				append_dev(div2, h20);
				append_dev(section0, t4);
				append_dev(section0, div4);
				append_dev(div4, div3);
				append_dev(div3, label0);
				append_dev(div3, t6);
				append_dev(div3, p0);
				append_dev(p0, t7);
				append_dev(main, t8);
				append_dev(main, section1);
				append_dev(section1, div5);
				append_dev(div5, h21);
				append_dev(section1, t10);
				if (if_block0) if_block0.m(section1, null);
				append_dev(section1, t11);
				append_dev(section1, div14);
				append_dev(div14, div9);
				append_dev(div9, div8);
				append_dev(div8, div6);
				append_dev(div6, svg1);
				append_dev(svg1, path1);
				append_dev(div8, t12);
				append_dev(div8, div7);
				append_dev(div7, p1);
				append_dev(div7, t14);
				if_block1.m(div7, null);
				append_dev(div9, t15);
				if_block2.m(div9, null);
				append_dev(div14, t16);
				append_dev(div14, div13);
				append_dev(div13, div12);
				append_dev(div12, div10);
				append_dev(div10, svg2);
				append_dev(svg2, path2);
				append_dev(div12, t17);
				append_dev(div12, div11);
				append_dev(div11, p2);
				append_dev(div11, t19);
				if_block3.m(div11, null);
				append_dev(div13, t20);
				if_block4.m(div13, null);
				append_dev(main, t21);
				append_dev(main, section2);
				append_dev(section2, div15);
				append_dev(div15, h22);
				append_dev(section2, t23);
				append_dev(section2, div25);
				append_dev(div25, div18);
				append_dev(div18, div16);
				append_dev(div16, p3);
				append_dev(div16, t25);
				append_dev(div16, p4);
				append_dev(div18, t27);
				append_dev(div18, label1);
				append_dev(label1, input0);
				append_dev(label1, t28);
				append_dev(label1, div17);
				append_dev(div25, t29);
				append_dev(div25, div21);
				append_dev(div21, div19);
				append_dev(div19, p5);
				append_dev(div19, t31);
				append_dev(div19, p6);
				append_dev(div21, t33);
				append_dev(div21, label2);
				append_dev(label2, input1);
				append_dev(label2, t34);
				append_dev(label2, div20);
				append_dev(div25, t35);
				append_dev(div25, div24);
				append_dev(div24, div22);
				append_dev(div22, p7);
				append_dev(div22, t37);
				append_dev(div22, p8);
				append_dev(div24, t39);
				append_dev(div24, label3);
				append_dev(label3, input2);
				append_dev(label3, t40);
				append_dev(label3, div23);
				append_dev(main, t41);
				append_dev(main, section3);
				append_dev(section3, div26);
				append_dev(div26, h23);
				append_dev(section3, t43);
				append_dev(section3, div27);
				append_dev(div27, button1);
				if_block5.m(button1, null);
				append_dev(div27, t44);
				append_dev(div27, button2);
				append_dev(main, t46);
				append_dev(main, p9);

				if (!mounted) {
					dispose = [
						listen_dev(button0, "click", /*click_handler*/ ctx[9], false, false, false, false),
						listen_dev(button1, "click", /*handleLogout*/ ctx[8], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, [dirty]) {
				if (dirty & /*$currentUser*/ 16 && t7_value !== (t7_value = (/*$currentUser*/ ctx[4]?.email || 'Not signed in') + "")) set_data_dev(t7, t7_value);

				if (/*connectionError*/ ctx[3]) {
					if (if_block0) {
						if_block0.p(ctx, dirty);
					} else {
						if_block0 = create_if_block_7(ctx);
						if_block0.c();
						if_block0.m(section1, t11);
					}
				} else if (if_block0) {
					if_block0.d(1);
					if_block0 = null;
				}

				if (current_block_type !== (current_block_type = select_block_type(ctx))) {
					if_block1.d(1);
					if_block1 = current_block_type(ctx);

					if (if_block1) {
						if_block1.c();
						if_block1.m(div7, null);
					}
				}

				if_block2.p(ctx, dirty);

				if (current_block_type_2 !== (current_block_type_2 = select_block_type_2(ctx))) {
					if_block3.d(1);
					if_block3 = current_block_type_2(ctx);

					if (if_block3) {
						if_block3.c();
						if_block3.m(div11, null);
					}
				}

				if_block4.p(ctx, dirty);

				if (current_block_type_4 !== (current_block_type_4 = select_block_type_4(ctx))) {
					if_block5.d(1);
					if_block5 = current_block_type_4(ctx);

					if (if_block5) {
						if_block5.c();
						if_block5.m(button1, null);
					}
				}

				if (dirty & /*isLoggingOut*/ 1) {
					prop_dev(button1, "disabled", /*isLoggingOut*/ ctx[0]);
				}
			},
			i: noop,
			o: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div28);
				}

				if (if_block0) if_block0.d();
				if_block1.d();
				if_block2.d();
				if_block3.d();
				if_block4.d();
				if_block5.d();
				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_fragment$2.name,
			type: "component",
			source: "",
			ctx
		});

		return block;
	}

	function instance$2($$self, $$props, $$invalidate) {
		let $currentUser;
		validate_store(currentUser, 'currentUser');
		component_subscribe($$self, currentUser, $$value => $$invalidate(4, $currentUser = $$value));
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('Settings', slots, []);
		let isLoggingOut = false;
		let connectedAccounts = [];
		let isLoadingConnections = true;
		let connectingProvider = null;
		let connectionError = null;

		onMount(async () => {
			await loadConnections();
		});

		async function loadConnections() {
			$$invalidate(1, isLoadingConnections = true);

			try {
				const result = await apiClient.get('/api/v1/auth/oauth/accounts');

				if (result.success && result.data) {
					connectedAccounts = result.data;
				}
			} catch(e) {
				console.error('Failed to load connections:', e);
			} finally {
				$$invalidate(1, isLoadingConnections = false);
			}
		}

		function isConnected(provider) {
			return connectedAccounts.some(a => a.provider === provider);
		}

		function getConnectionInfo(provider) {
			return connectedAccounts.find(a => a.provider === provider);
		}

		async function initiateOAuth(provider) {
			$$invalidate(2, connectingProvider = provider);
			$$invalidate(3, connectionError = null);

			try {
				const result = await apiClient.post(`/api/v1/auth/oauth/${provider}/link`);

				if (result.success && result.data?.auth_url) {
					// Redirect to OAuth provider
					window.location.href = result.data.auth_url;
				} else {
					$$invalidate(3, connectionError = result.error || 'Failed to initiate connection');
				}
			} catch(e) {
				console.error('OAuth initiation failed:', e);
				$$invalidate(3, connectionError = 'Failed to connect. Please try again.');
			} finally {
				$$invalidate(2, connectingProvider = null);
			}
		}

		async function disconnectService(provider) {
			$$invalidate(2, connectingProvider = provider);

			try {
				const result = await apiClient.delete(`/api/v1/auth/oauth/${provider}/unlink`);

				if (result.success) {
					connectedAccounts = connectedAccounts.filter(a => a.provider !== provider);
				}
			} catch(e) {
				console.error('Disconnect failed:', e);
			} finally {
				$$invalidate(2, connectingProvider = null);
			}
		}

		async function handleLogout() {
			$$invalidate(0, isLoggingOut = true);

			try {
				await authActions.logout();
				window.location.href = '/';
			} catch(e) {
				console.error('Logout failed:', e);
				$$invalidate(0, isLoggingOut = false);
			}
		}

		const writable_props = [];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console_1$1.warn(`<Settings> was created with unknown prop '${key}'`);
		});

		const click_handler = () => navigateTo('home');
		const click_handler_1 = () => disconnectService('spotify');
		const click_handler_2 = () => initiateOAuth('spotify');
		const click_handler_3 = () => disconnectService('apple');
		const click_handler_4 = () => initiateOAuth('apple');

		$$self.$capture_state = () => ({
			onMount,
			currentUser,
			authActions,
			navigateTo,
			apiClient,
			isLoggingOut,
			connectedAccounts,
			isLoadingConnections,
			connectingProvider,
			connectionError,
			loadConnections,
			isConnected,
			getConnectionInfo,
			initiateOAuth,
			disconnectService,
			handleLogout,
			$currentUser
		});

		$$self.$inject_state = $$props => {
			if ('isLoggingOut' in $$props) $$invalidate(0, isLoggingOut = $$props.isLoggingOut);
			if ('connectedAccounts' in $$props) connectedAccounts = $$props.connectedAccounts;
			if ('isLoadingConnections' in $$props) $$invalidate(1, isLoadingConnections = $$props.isLoadingConnections);
			if ('connectingProvider' in $$props) $$invalidate(2, connectingProvider = $$props.connectingProvider);
			if ('connectionError' in $$props) $$invalidate(3, connectionError = $$props.connectionError);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		return [
			isLoggingOut,
			isLoadingConnections,
			connectingProvider,
			connectionError,
			$currentUser,
			isConnected,
			initiateOAuth,
			disconnectService,
			handleLogout,
			click_handler,
			click_handler_1,
			click_handler_2,
			click_handler_3,
			click_handler_4
		];
	}

	class Settings extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance$2, create_fragment$2, safe_not_equal, {});

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "Settings",
				options,
				id: create_fragment$2.name
			});
		}
	}

	/* src/lib/components/OAuthCallback.svelte generated by Svelte v4.2.20 */

	const { Error: Error_1 } = globals;
	const file$1 = "src/lib/components/OAuthCallback.svelte";

	// (92:4) {:else}
	function create_else_block$1(ctx) {
		let div1;
		let div0;
		let svg;
		let path;
		let t0;
		let h1;
		let t2;
		let p;
		let t3;
		let t4;
		let div2;
		let button0;
		let t6;
		let button1;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				div1 = element("div");
				div0 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				h1 = element("h1");
				h1.textContent = "Connection Failed";
				t2 = space();
				p = element("p");
				t3 = text(/*errorMessage*/ ctx[1]);
				t4 = space();
				div2 = element("div");
				button0 = element("button");
				button0.textContent = "Try Again";
				t6 = space();
				button1 = element("button");
				button1.textContent = "Go Home";
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M6 18L18 6M6 6l12 12");
				add_location(path, file$1, 104, 12, 3624);
				attr_dev(svg, "class", "w-10 h-10 text-white");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "stroke", "currentColor");
				attr_dev(svg, "viewBox", "0 0 24 24");
				add_location(svg, file$1, 103, 10, 3523);
				attr_dev(div0, "class", "w-16 h-16 bg-red-500 rounded-full flex items-center justify-center mx-auto");
				add_location(div0, file$1, 102, 8, 3424);
				attr_dev(div1, "class", "mb-6");
				add_location(div1, file$1, 101, 6, 3397);
				attr_dev(h1, "class", "text-xl font-bold mb-2 text-red-400");
				add_location(h1, file$1, 108, 6, 3773);
				attr_dev(p, "class", "text-gray-400 mb-6");
				add_location(p, file$1, 109, 6, 3850);
				attr_dev(button0, "class", "px-6 py-3 bg-gray-700 hover:bg-gray-600 rounded-lg font-medium transition-colors");
				add_location(button0, file$1, 111, 8, 3953);
				attr_dev(button1, "class", "px-6 py-3 bg-rose-600 hover:bg-rose-700 rounded-lg font-medium transition-colors");
				add_location(button1, file$1, 117, 8, 4150);
				attr_dev(div2, "class", "flex gap-3 justify-center");
				add_location(div2, file$1, 110, 6, 3905);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div1, anchor);
				append_dev(div1, div0);
				append_dev(div0, svg);
				append_dev(svg, path);
				insert_dev(target, t0, anchor);
				insert_dev(target, h1, anchor);
				insert_dev(target, t2, anchor);
				insert_dev(target, p, anchor);
				append_dev(p, t3);
				insert_dev(target, t4, anchor);
				insert_dev(target, div2, anchor);
				append_dev(div2, button0);
				append_dev(div2, t6);
				append_dev(div2, button1);

				if (!mounted) {
					dispose = [
						listen_dev(button0, "click", /*goToSettings*/ ctx[3], false, false, false, false),
						listen_dev(button1, "click", /*goHome*/ ctx[4], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (dirty & /*errorMessage*/ 2) set_data_dev(t3, /*errorMessage*/ ctx[1]);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div1);
					detach_dev(t0);
					detach_dev(h1);
					detach_dev(t2);
					detach_dev(p);
					detach_dev(t4);
					detach_dev(div2);
				}

				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block$1.name,
			type: "else",
			source: "(92:4) {:else}",
			ctx
		});

		return block;
	}

	// (76:35) 
	function create_if_block_1$1(ctx) {
		let div1;
		let div0;
		let svg;
		let path;
		let t0;
		let h1;
		let t2;
		let p;
		let t3;
		let t4_value = getProviderName(/*provider*/ ctx[2]) + "";
		let t4;
		let t5;
		let t6;
		let button;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				div1 = element("div");
				div0 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				h1 = element("h1");
				h1.textContent = "Connected!";
				t2 = space();
				p = element("p");
				t3 = text("Your ");
				t4 = text(t4_value);
				t5 = text(" account has been linked successfully.");
				t6 = space();
				button = element("button");
				button.textContent = "Go to Settings";
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M5 13l4 4L19 7");
				add_location(path, file$1, 88, 12, 2869);
				attr_dev(svg, "class", "w-10 h-10 text-white");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "stroke", "currentColor");
				attr_dev(svg, "viewBox", "0 0 24 24");
				add_location(svg, file$1, 87, 10, 2768);
				attr_dev(div0, "class", "w-16 h-16 bg-green-500 rounded-full flex items-center justify-center mx-auto");
				add_location(div0, file$1, 86, 8, 2667);
				attr_dev(div1, "class", "mb-6");
				add_location(div1, file$1, 85, 6, 2640);
				attr_dev(h1, "class", "text-xl font-bold mb-2 text-green-400");
				add_location(h1, file$1, 92, 6, 3012);
				attr_dev(p, "class", "text-gray-400 mb-6");
				add_location(p, file$1, 93, 6, 3084);
				attr_dev(button, "class", "px-6 py-3 bg-rose-600 hover:bg-rose-700 rounded-lg font-medium transition-colors");
				add_location(button, file$1, 94, 6, 3195);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div1, anchor);
				append_dev(div1, div0);
				append_dev(div0, svg);
				append_dev(svg, path);
				insert_dev(target, t0, anchor);
				insert_dev(target, h1, anchor);
				insert_dev(target, t2, anchor);
				insert_dev(target, p, anchor);
				append_dev(p, t3);
				append_dev(p, t4);
				append_dev(p, t5);
				insert_dev(target, t6, anchor);
				insert_dev(target, button, anchor);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*goToSettings*/ ctx[3], false, false, false, false);
					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (dirty & /*provider*/ 4 && t4_value !== (t4_value = getProviderName(/*provider*/ ctx[2]) + "")) set_data_dev(t4, t4_value);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div1);
					detach_dev(t0);
					detach_dev(h1);
					detach_dev(t2);
					detach_dev(p);
					detach_dev(t6);
					detach_dev(button);
				}

				mounted = false;
				dispose();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_1$1.name,
			type: "if",
			source: "(76:35) ",
			ctx
		});

		return block;
	}

	// (70:4) {#if status === 'loading'}
	function create_if_block$1(ctx) {
		let div1;
		let div0;
		let t0;
		let h1;
		let t1;
		let t2_value = getProviderName(/*provider*/ ctx[2]) + "";
		let t2;
		let t3;
		let p;

		const block = {
			c: function create() {
				div1 = element("div");
				div0 = element("div");
				t0 = space();
				h1 = element("h1");
				t1 = text("Connecting ");
				t2 = text(t2_value);
				t3 = space();
				p = element("p");
				p.textContent = "Please wait while we complete the connection...";
				attr_dev(div0, "class", "w-16 h-16 border-4 border-rose-500 border-t-transparent rounded-full animate-spin mx-auto");
				add_location(div0, file$1, 80, 8, 2307);
				attr_dev(div1, "class", "mb-6");
				add_location(div1, file$1, 79, 6, 2280);
				attr_dev(h1, "class", "text-xl font-bold mb-2");
				add_location(h1, file$1, 82, 6, 2436);
				attr_dev(p, "class", "text-gray-400");
				add_location(p, file$1, 83, 6, 2521);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div1, anchor);
				append_dev(div1, div0);
				insert_dev(target, t0, anchor);
				insert_dev(target, h1, anchor);
				append_dev(h1, t1);
				append_dev(h1, t2);
				insert_dev(target, t3, anchor);
				insert_dev(target, p, anchor);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*provider*/ 4 && t2_value !== (t2_value = getProviderName(/*provider*/ ctx[2]) + "")) set_data_dev(t2, t2_value);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div1);
					detach_dev(t0);
					detach_dev(h1);
					detach_dev(t3);
					detach_dev(p);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block$1.name,
			type: "if",
			source: "(70:4) {#if status === 'loading'}",
			ctx
		});

		return block;
	}

	function create_fragment$1(ctx) {
		let div1;
		let div0;

		function select_block_type(ctx, dirty) {
			if (/*status*/ ctx[0] === 'loading') return create_if_block$1;
			if (/*status*/ ctx[0] === 'success') return create_if_block_1$1;
			return create_else_block$1;
		}

		let current_block_type = select_block_type(ctx);
		let if_block = current_block_type(ctx);

		const block = {
			c: function create() {
				div1 = element("div");
				div0 = element("div");
				if_block.c();
				attr_dev(div0, "class", "bg-gray-800 rounded-xl border border-gray-600 p-8 max-w-md w-full text-center");
				add_location(div0, file$1, 77, 2, 2151);
				attr_dev(div1, "class", "min-h-screen bg-gray-900 text-white flex items-center justify-center p-4");
				add_location(div1, file$1, 76, 0, 2062);
			},
			l: function claim(nodes) {
				throw new Error_1("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				insert_dev(target, div1, anchor);
				append_dev(div1, div0);
				if_block.m(div0, null);
			},
			p: function update(ctx, [dirty]) {
				if (current_block_type === (current_block_type = select_block_type(ctx)) && if_block) {
					if_block.p(ctx, dirty);
				} else {
					if_block.d(1);
					if_block = current_block_type(ctx);

					if (if_block) {
						if_block.c();
						if_block.m(div0, null);
					}
				}
			},
			i: noop,
			o: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div1);
				}

				if_block.d();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_fragment$1.name,
			type: "component",
			source: "",
			ctx
		});

		return block;
	}

	function getProviderName(p) {
		switch (p) {
			case 'spotify':
				return 'Spotify';
			case 'apple':
				return 'Apple Music';
			case 'google':
				return 'Google';
			case 'github':
				return 'GitHub';
			default:
				return p;
		}
	}

	function instance$1($$self, $$props, $$invalidate) {
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('OAuthCallback', slots, []);
		let status = 'loading';
		let errorMessage = '';
		let provider = '';

		onMount(async () => {
			// Parse URL parameters
			const params = new URLSearchParams(window.location.search);

			const code = params.get('code');
			const state = params.get('state');
			const error = params.get('error');
			const errorDescription = params.get('error_description');

			// Extract provider from path (e.g., /auth/callback/spotify)
			const pathParts = window.location.pathname.split('/');

			$$invalidate(2, provider = pathParts[pathParts.length - 1] || 'unknown');

			// Handle OAuth errors
			if (error) {
				$$invalidate(0, status = 'error');
				$$invalidate(1, errorMessage = errorDescription || error || 'Authentication was cancelled or denied');
				return;
			}

			if (!code || !state) {
				$$invalidate(0, status = 'error');
				$$invalidate(1, errorMessage = 'Missing authentication parameters');
				return;
			}

			try {
				// Complete the OAuth link flow
				const result = await apiClient.post(`/api/v1/auth/oauth/${provider}/link-callback`, { code, state });

				if (result.success) {
					$$invalidate(0, status = 'success');

					// Redirect to settings after a brief moment
					setTimeout(
						() => {
							navigateTo('settings');
						},
						1500
					);
				} else {
					$$invalidate(0, status = 'error');
					$$invalidate(1, errorMessage = result.error || 'Failed to link account');
				}
			} catch(e) {
				$$invalidate(0, status = 'error');

				$$invalidate(1, errorMessage = e instanceof Error
				? e.message
				: 'An unexpected error occurred');
			}
		});

		function goToSettings() {
			navigateTo('settings');
		}

		function goHome() {
			navigateTo('home');
		}

		const writable_props = [];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<OAuthCallback> was created with unknown prop '${key}'`);
		});

		$$self.$capture_state = () => ({
			onMount,
			apiClient,
			navigateTo,
			status,
			errorMessage,
			provider,
			goToSettings,
			goHome,
			getProviderName
		});

		$$self.$inject_state = $$props => {
			if ('status' in $$props) $$invalidate(0, status = $$props.status);
			if ('errorMessage' in $$props) $$invalidate(1, errorMessage = $$props.errorMessage);
			if ('provider' in $$props) $$invalidate(2, provider = $$props.provider);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		return [status, errorMessage, provider, goToSettings, goHome];
	}

	class OAuthCallback extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance$1, create_fragment$1, safe_not_equal, {});

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "OAuthCallback",
				options,
				id: create_fragment$1.name
			});
		}
	}

	/* src/App.svelte generated by Svelte v4.2.20 */

	const { console: console_1 } = globals;
	const file = "src/App.svelte";

	// (52:0) {:else}
	function create_else_block_1(ctx) {
		let login;
		let current;
		login = new Login({ $$inline: true });

		const block = {
			c: function create() {
				create_component(login.$$.fragment);
			},
			m: function mount(target, anchor) {
				mount_component(login, target, anchor);
				current = true;
			},
			p: noop,
			i: function intro(local) {
				if (current) return;
				transition_in(login.$$.fragment, local);
				current = true;
			},
			o: function outro(local) {
				transition_out(login.$$.fragment, local);
				current = false;
			},
			d: function destroy(detaching) {
				destroy_component(login, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block_1.name,
			type: "else",
			source: "(52:0) {:else}",
			ctx
		});

		return block;
	}

	// (46:27) 
	function create_if_block_3(ctx) {
		let current_block_type_index;
		let if_block;
		let if_block_anchor;
		let current;
		const if_block_creators = [create_if_block_4, create_else_block];
		const if_blocks = [];

		function select_block_type_1(ctx, dirty) {
			if (/*$currentRoute*/ ctx[1] === 'settings') return 0;
			return 1;
		}

		current_block_type_index = select_block_type_1(ctx);
		if_block = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);

		const block = {
			c: function create() {
				if_block.c();
				if_block_anchor = empty();
			},
			m: function mount(target, anchor) {
				if_blocks[current_block_type_index].m(target, anchor);
				insert_dev(target, if_block_anchor, anchor);
				current = true;
			},
			p: function update(ctx, dirty) {
				let previous_block_index = current_block_type_index;
				current_block_type_index = select_block_type_1(ctx);

				if (current_block_type_index !== previous_block_index) {
					group_outros();

					transition_out(if_blocks[previous_block_index], 1, 1, () => {
						if_blocks[previous_block_index] = null;
					});

					check_outros();
					if_block = if_blocks[current_block_type_index];

					if (!if_block) {
						if_block = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);
						if_block.c();
					}

					transition_in(if_block, 1);
					if_block.m(if_block_anchor.parentNode, if_block_anchor);
				}
			},
			i: function intro(local) {
				if (current) return;
				transition_in(if_block);
				current = true;
			},
			o: function outro(local) {
				transition_out(if_block);
				current = false;
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(if_block_anchor);
				}

				if_blocks[current_block_type_index].d(detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_3.name,
			type: "if",
			source: "(46:27) ",
			ctx
		});

		return block;
	}

	// (32:42) 
	function create_if_block_2(ctx) {
		let div2;
		let div1;
		let div0;
		let t1;
		let h2;
		let t3;
		let p;
		let t5;
		let button;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				div2 = element("div");
				div1 = element("div");
				div0 = element("div");
				div0.textContent = "!";
				t1 = space();
				h2 = element("h2");
				h2.textContent = "Connection Failed";
				t3 = space();
				p = element("p");
				p.textContent = "There was a problem connecting your account. Please try again.";
				t5 = space();
				button = element("button");
				button.textContent = "Go Back";
				attr_dev(div0, "class", "text-red-500 text-6xl mb-4");
				add_location(div0, file, 35, 3, 1213);
				attr_dev(h2, "class", "text-2xl font-bold text-white mb-2");
				add_location(h2, file, 36, 3, 1264);
				attr_dev(p, "class", "text-gray-400 mb-6");
				add_location(p, file, 37, 3, 1337);
				attr_dev(button, "class", "px-6 py-3 bg-red-600 hover:bg-red-700 text-white rounded-lg font-medium transition-colors");
				add_location(button, file, 38, 3, 1437);
				attr_dev(div1, "class", "max-w-md w-full text-center");
				add_location(div1, file, 34, 2, 1168);
				attr_dev(div2, "class", "min-h-screen flex items-center justify-center bg-gray-900 py-12 px-4");
				add_location(div2, file, 33, 1, 1083);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div2, anchor);
				append_dev(div2, div1);
				append_dev(div1, div0);
				append_dev(div1, t1);
				append_dev(div1, h2);
				append_dev(div1, t3);
				append_dev(div1, p);
				append_dev(div1, t5);
				append_dev(div1, button);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*click_handler*/ ctx[3], false, false, false, false);
					mounted = true;
				}
			},
			p: noop,
			i: noop,
			o: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div2);
				}

				mounted = false;
				dispose();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_2.name,
			type: "if",
			source: "(32:42) ",
			ctx
		});

		return block;
	}

	// (30:45) 
	function create_if_block_1(ctx) {
		let oauthcallback;
		let current;
		oauthcallback = new OAuthCallback({ $$inline: true });

		const block = {
			c: function create() {
				create_component(oauthcallback.$$.fragment);
			},
			m: function mount(target, anchor) {
				mount_component(oauthcallback, target, anchor);
				current = true;
			},
			p: noop,
			i: function intro(local) {
				if (current) return;
				transition_in(oauthcallback.$$.fragment, local);
				current = true;
			},
			o: function outro(local) {
				transition_out(oauthcallback.$$.fragment, local);
				current = false;
			},
			d: function destroy(detaching) {
				destroy_component(oauthcallback, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_1.name,
			type: "if",
			source: "(30:45) ",
			ctx
		});

		return block;
	}

	// (23:0) {#if !isInitialized}
	function create_if_block(ctx) {
		let div2;
		let div1;
		let div0;
		let t0;
		let p;

		const block = {
			c: function create() {
				div2 = element("div");
				div1 = element("div");
				div0 = element("div");
				t0 = space();
				p = element("p");
				p.textContent = "Loading...";
				attr_dev(div0, "class", "w-8 h-8 border-4 border-red-500 border-t-transparent rounded-full animate-spin mx-auto");
				add_location(div0, file, 26, 3, 802);
				attr_dev(p, "class", "mt-4 text-gray-400");
				add_location(p, file, 27, 3, 912);
				attr_dev(div1, "class", "text-center");
				add_location(div1, file, 25, 2, 773);
				attr_dev(div2, "class", "min-h-screen flex items-center justify-center bg-gray-900");
				add_location(div2, file, 24, 1, 699);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div2, anchor);
				append_dev(div2, div1);
				append_dev(div1, div0);
				append_dev(div1, t0);
				append_dev(div1, p);
			},
			p: noop,
			i: noop,
			o: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div2);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block.name,
			type: "if",
			source: "(23:0) {#if !isInitialized}",
			ctx
		});

		return block;
	}

	// (49:1) {:else}
	function create_else_block(ctx) {
		let home;
		let current;
		home = new Home({ $$inline: true });

		const block = {
			c: function create() {
				create_component(home.$$.fragment);
			},
			m: function mount(target, anchor) {
				mount_component(home, target, anchor);
				current = true;
			},
			i: function intro(local) {
				if (current) return;
				transition_in(home.$$.fragment, local);
				current = true;
			},
			o: function outro(local) {
				transition_out(home.$$.fragment, local);
				current = false;
			},
			d: function destroy(detaching) {
				destroy_component(home, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block.name,
			type: "else",
			source: "(49:1) {:else}",
			ctx
		});

		return block;
	}

	// (47:1) {#if $currentRoute === 'settings'}
	function create_if_block_4(ctx) {
		let settings;
		let current;
		settings = new Settings({ $$inline: true });

		const block = {
			c: function create() {
				create_component(settings.$$.fragment);
			},
			m: function mount(target, anchor) {
				mount_component(settings, target, anchor);
				current = true;
			},
			i: function intro(local) {
				if (current) return;
				transition_in(settings.$$.fragment, local);
				current = true;
			},
			o: function outro(local) {
				transition_out(settings.$$.fragment, local);
				current = false;
			},
			d: function destroy(detaching) {
				destroy_component(settings, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_4.name,
			type: "if",
			source: "(47:1) {#if $currentRoute === 'settings'}",
			ctx
		});

		return block;
	}

	function create_fragment(ctx) {
		let current_block_type_index;
		let if_block;
		let if_block_anchor;
		let current;

		const if_block_creators = [
			create_if_block,
			create_if_block_1,
			create_if_block_2,
			create_if_block_3,
			create_else_block_1
		];

		const if_blocks = [];

		function select_block_type(ctx, dirty) {
			if (!/*isInitialized*/ ctx[0]) return 0;
			if (/*$currentRoute*/ ctx[1] === 'oauth-callback') return 1;
			if (/*$currentRoute*/ ctx[1] === 'oauth-error') return 2;
			if (/*$isAuthenticated*/ ctx[2]) return 3;
			return 4;
		}

		current_block_type_index = select_block_type(ctx);
		if_block = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);

		const block = {
			c: function create() {
				if_block.c();
				if_block_anchor = empty();
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				if_blocks[current_block_type_index].m(target, anchor);
				insert_dev(target, if_block_anchor, anchor);
				current = true;
			},
			p: function update(ctx, [dirty]) {
				let previous_block_index = current_block_type_index;
				current_block_type_index = select_block_type(ctx);

				if (current_block_type_index === previous_block_index) {
					if_blocks[current_block_type_index].p(ctx, dirty);
				} else {
					group_outros();

					transition_out(if_blocks[previous_block_index], 1, 1, () => {
						if_blocks[previous_block_index] = null;
					});

					check_outros();
					if_block = if_blocks[current_block_type_index];

					if (!if_block) {
						if_block = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);
						if_block.c();
					} else {
						if_block.p(ctx, dirty);
					}

					transition_in(if_block, 1);
					if_block.m(if_block_anchor.parentNode, if_block_anchor);
				}
			},
			i: function intro(local) {
				if (current) return;
				transition_in(if_block);
				current = true;
			},
			o: function outro(local) {
				transition_out(if_block);
				current = false;
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(if_block_anchor);
				}

				if_blocks[current_block_type_index].d(detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_fragment.name,
			type: "component",
			source: "",
			ctx
		});

		return block;
	}

	function instance($$self, $$props, $$invalidate) {
		let $currentRoute;
		let $isAuthenticated;
		validate_store(currentRoute, 'currentRoute');
		component_subscribe($$self, currentRoute, $$value => $$invalidate(1, $currentRoute = $$value));
		validate_store(isAuthenticated, 'isAuthenticated');
		component_subscribe($$self, isAuthenticated, $$value => $$invalidate(2, $isAuthenticated = $$value));
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('App', slots, []);
		let isInitialized = false;

		onMount(async () => {
			try {
				initRouter();
				await authActions.fetchProfile();
			} catch(error) {
				console.error("Init error:", error);
			} finally {
				$$invalidate(0, isInitialized = true);
			}
		});

		const writable_props = [];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console_1.warn(`<App> was created with unknown prop '${key}'`);
		});

		const click_handler = () => window.location.href = '/';

		$$self.$capture_state = () => ({
			onMount,
			isAuthenticated,
			authActions,
			initRouter,
			currentRoute,
			Login,
			Home,
			Settings,
			OAuthCallback,
			isInitialized,
			$currentRoute,
			$isAuthenticated
		});

		$$self.$inject_state = $$props => {
			if ('isInitialized' in $$props) $$invalidate(0, isInitialized = $$props.isInitialized);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		return [isInitialized, $currentRoute, $isAuthenticated, click_handler];
	}

	class App extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance, create_fragment, safe_not_equal, {});

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "App",
				options,
				id: create_fragment.name
			});
		}
	}

	const app = new App({
	    target: document.body,
	});

	return app;

})();
//# sourceMappingURL=bundle.js.map
