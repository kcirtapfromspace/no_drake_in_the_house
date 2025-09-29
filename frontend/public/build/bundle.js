
(function(l, r) { if (!l || l.getElementById('livereloadscript')) return; r = l.createElement('script'); r.async = 1; r.src = '//' + (self.location.host || 'localhost').split(':')[0] + ':35730/livereload.js?snipver=1'; r.id = 'livereloadscript'; l.getElementsByTagName('head')[0].appendChild(r) })(self.document);
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
	 * @param {HTMLInputElement[]} group
	 * @returns {{ p(...inputs: HTMLInputElement[]): void; r(): void; }}
	 */
	function init_binding_group(group) {
		/**
		 * @type {HTMLInputElement[]} */
		let _inputs;
		return {
			/* push */ p(...inputs) {
				_inputs = inputs;
				_inputs.forEach((input) => group.push(input));
			},
			/* remove */ r() {
				_inputs.forEach((input) => group.splice(group.indexOf(input), 1));
			}
		};
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
	 * @returns {void} */
	function set_style(node, key, value, important) {
		if (value == null) {
			node.style.removeProperty(key);
		} else {
			node.style.setProperty(key, value, '');
		}
	}

	/**
	 * @returns {void} */
	function select_option(select, value, mounting) {
		for (let i = 0; i < select.options.length; i += 1) {
			const option = select.options[i];
			if (option.__value === value) {
				option.selected = true;
				return;
			}
		}
		if (!mounting || value !== undefined) {
			select.selectedIndex = -1; // no option should be selected
		}
	}

	function select_value(select) {
		const selected_option = select.querySelector(':checked');
		return selected_option && selected_option.__value;
	}

	/**
	 * @returns {void} */
	function toggle_class(element, name, toggle) {
		// The `!!` is required because an `undefined` flag means flipping the current state.
		element.classList.toggle(name, !!toggle);
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

	/**
	 * Creates an event dispatcher that can be used to dispatch [component events](https://svelte.dev/docs#template-syntax-component-directives-on-eventname).
	 * Event dispatchers are functions that can take two arguments: `name` and `detail`.
	 *
	 * Component events created with `createEventDispatcher` create a
	 * [CustomEvent](https://developer.mozilla.org/en-US/docs/Web/API/CustomEvent).
	 * These events do not [bubble](https://developer.mozilla.org/en-US/docs/Learn/JavaScript/Building_blocks/Events#Event_bubbling_and_capture).
	 * The `detail` argument corresponds to the [CustomEvent.detail](https://developer.mozilla.org/en-US/docs/Web/API/CustomEvent/detail)
	 * property and can contain any type of data.
	 *
	 * The event dispatcher can be typed to narrow the allowed event names and the type of the `detail` argument:
	 * ```ts
	 * const dispatch = createEventDispatcher<{
	 *  loaded: never; // does not take a detail argument
	 *  change: string; // takes a detail argument of type string, which is required
	 *  optional: number | null; // takes an optional detail argument of type number
	 * }>();
	 * ```
	 *
	 * https://svelte.dev/docs/svelte#createeventdispatcher
	 * @template {Record<string, any>} [EventMap=any]
	 * @returns {import('./public.js').EventDispatcher<EventMap>}
	 */
	function createEventDispatcher() {
		const component = get_current_component();
		return (type, detail, { cancelable = false } = {}) => {
			const callbacks = component.$$.callbacks[type];
			if (callbacks) {
				// TODO are there situations where events could be dispatched
				// in a server (non-DOM) environment?
				const event = custom_event(/** @type {string} */ (type), detail, { cancelable });
				callbacks.slice().forEach((fn) => {
					fn.call(component, event);
				});
				return !event.defaultPrevented;
			}
			return true;
		};
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
	function outro_and_destroy_block(block, lookup) {
		transition_out(block, 1, 1, () => {
			lookup.delete(block.key);
		});
	}

	/** @returns {any[]} */
	function update_keyed_each(
		old_blocks,
		dirty,
		get_key,
		dynamic,
		ctx,
		list,
		lookup,
		node,
		destroy,
		create_each_block,
		next,
		get_context
	) {
		let o = old_blocks.length;
		let n = list.length;
		let i = o;
		const old_indexes = {};
		while (i--) old_indexes[old_blocks[i].key] = i;
		const new_blocks = [];
		const new_lookup = new Map();
		const deltas = new Map();
		const updates = [];
		i = n;
		while (i--) {
			const child_ctx = get_context(ctx, list, i);
			const key = get_key(child_ctx);
			let block = lookup.get(key);
			if (!block) {
				block = create_each_block(key, child_ctx);
				block.c();
			} else {
				// defer updates until all the DOM shuffling is done
				updates.push(() => block.p(child_ctx, dirty));
			}
			new_lookup.set(key, (new_blocks[i] = block));
			if (key in old_indexes) deltas.set(key, Math.abs(i - old_indexes[key]));
		}
		const will_move = new Set();
		const did_move = new Set();
		/** @returns {void} */
		function insert(block) {
			transition_in(block, 1);
			block.m(node, next);
			lookup.set(block.key, block);
			next = block.first;
			n--;
		}
		while (o && n) {
			const new_block = new_blocks[n - 1];
			const old_block = old_blocks[o - 1];
			const new_key = new_block.key;
			const old_key = old_block.key;
			if (new_block === old_block) {
				// do nothing
				next = new_block.first;
				o--;
				n--;
			} else if (!new_lookup.has(old_key)) {
				// remove old block
				destroy(old_block, lookup);
				o--;
			} else if (!lookup.has(new_key) || will_move.has(new_key)) {
				insert(new_block);
			} else if (did_move.has(old_key)) {
				o--;
			} else if (deltas.get(new_key) > deltas.get(old_key)) {
				did_move.add(new_key);
				insert(new_block);
			} else {
				will_move.add(old_key);
				o--;
			}
		}
		while (o--) {
			const old_block = old_blocks[o];
			if (!new_lookup.has(old_block.key)) destroy(old_block, lookup);
		}
		while (n) insert(new_blocks[n - 1]);
		run_all(updates);
		return new_blocks;
	}

	/** @returns {void} */
	function validate_each_keys(ctx, list, get_context, get_key) {
		const keys = new Map();
		for (let i = 0; i < list.length; i++) {
			const key = get_key(get_context(ctx, list, i));
			if (keys.has(key)) {
				let value = '';
				try {
					value = `with value '${String(key)}' `;
				} catch (e) {
					// can't stringify
				}
				throw new Error(
					`Cannot have duplicate keys in a keyed each: Keys at index ${keys.get(
					key
				)} and ${i} ${value}are duplicates`
				);
			}
			keys.set(key, i);
		}
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

	// Configuration utilities for environment-based settings
	const config = {
	    // API Configuration
	    apiUrl: undefined.VITE_API_URL || 'http://localhost:3000',
	    apiVersion: undefined.VITE_API_VERSION || 'v1',
	    // App Configuration
	    appName: undefined.VITE_APP_NAME || 'No Drake in the House',
	    environment: undefined.VITE_ENVIRONMENT || 'development',
	    // Feature Flags
	    features: {
	        twoFactorAuth: undefined.VITE_ENABLE_2FA === 'true',
	        communityLists: undefined.VITE_ENABLE_COMMUNITY_LISTS === 'true',
	        analytics: undefined.VITE_ENABLE_ANALYTICS === 'true',
	    },
	    // Development Configuration
	    development: {
	        hotReload: undefined.VITE_HOT_RELOAD === 'true',
	        debugMode: undefined.VITE_DEBUG_MODE === 'true',
	    },
	    // External Services
	    external: {
	        spotifyClientId: undefined.VITE_SPOTIFY_CLIENT_ID,
	        appleMusicToken: undefined.VITE_APPLE_MUSIC_DEVELOPER_TOKEN,
	    },
	    // UI Configuration
	    ui: {
	        defaultTheme: undefined.VITE_DEFAULT_THEME || 'light',
	    },
	    // Performance Configuration
	    performance: {
	        enableServiceWorker: undefined.VITE_ENABLE_SERVICE_WORKER === 'true',
	        cacheDuration: parseInt(undefined.VITE_CACHE_DURATION || '300000'),
	    },
	    // Helper methods
	    isDevelopment: () => config.environment === 'development',
	    isProduction: () => config.environment === 'production',
	    getApiEndpoint: (path) => {
	        const normalizedPath = path.startsWith('/') ? path : `/${path}`;
	        const apiPath = normalizedPath.startsWith('/api/') ? normalizedPath : `/api/${config.apiVersion}${normalizedPath}`;
	        return `${config.apiUrl}${apiPath}`;
	    }
	};

	class ApiError extends Error {
	    constructor(status, message) {
	        super(message);
	        Object.defineProperty(this, "status", {
	            enumerable: true,
	            configurable: true,
	            writable: true,
	            value: status
	        });
	        this.name = 'ApiError';
	    }
	}
	async function apiCall(endpoint, options = {}) {
	    const token = localStorage.getItem('auth_token');
	    const requestConfig = {
	        ...options,
	        headers: {
	            'Content-Type': 'application/json',
	            ...(token && { Authorization: `Bearer ${token}` }),
	            ...options.headers,
	        },
	    };
	    try {
	        const url = config.getApiEndpoint(endpoint);
	        const response = await fetch(url, requestConfig);
	        const result = await response.json();
	        if (!response.ok) {
	            throw new ApiError(response.status, result.message || 'Request failed');
	        }
	        return result;
	    }
	    catch (error) {
	        if (error instanceof ApiError) {
	            throw error;
	        }
	        throw new ApiError(0, 'Network error occurred');
	    }
	}
	const api = {
	    get: (endpoint) => apiCall(endpoint),
	    post: (endpoint, data) => apiCall(endpoint, { method: 'POST', body: JSON.stringify(data) }),
	    put: (endpoint, data) => apiCall(endpoint, { method: 'PUT', body: JSON.stringify(data) }),
	    delete: (endpoint, data) => apiCall(endpoint, {
	        method: 'DELETE',
	        body: data ? JSON.stringify(data) : undefined
	    }),
	};

	const initialState$4 = {
	    user: null,
	    token: localStorage.getItem('auth_token'),
	    refreshToken: localStorage.getItem('refresh_token'),
	    isAuthenticated: false,
	    isLoading: false,
	};
	const authStore = writable(initialState$4);
	const isAuthenticated = derived(authStore, ($auth) => $auth.isAuthenticated && $auth.token !== null);
	const currentUser = derived(authStore, ($auth) => $auth.user);
	// Auth actions
	const authActions = {
	    login: async (email, password, totpCode) => {
	        authStore.update(state => ({ ...state, isLoading: true }));
	        try {
	            const result = await api.post('/auth/login', {
	                email,
	                password,
	                totp_code: totpCode
	            });
	            if (result.success) {
	                const { access_token, refresh_token } = result.data;
	                localStorage.setItem('auth_token', access_token);
	                localStorage.setItem('refresh_token', refresh_token);
	                authStore.update(state => ({
	                    ...state,
	                    token: access_token,
	                    refreshToken: refresh_token,
	                    isAuthenticated: true,
	                    isLoading: false,
	                }));
	                // Fetch user profile
	                await authActions.fetchProfile();
	                return { success: true };
	            }
	            else {
	                authStore.update(state => ({ ...state, isLoading: false }));
	                return { success: false, message: result.message };
	            }
	        }
	        catch (error) {
	            authStore.update(state => ({ ...state, isLoading: false }));
	            return { success: false, message: error.message || 'Network error occurred' };
	        }
	    },
	    register: async (email, password) => {
	        authStore.update(state => ({ ...state, isLoading: true }));
	        try {
	            const result = await api.post('/auth/register', { email, password });
	            authStore.update(state => ({ ...state, isLoading: false }));
	            return { success: result.success, message: result.message };
	        }
	        catch (error) {
	            authStore.update(state => ({ ...state, isLoading: false }));
	            return { success: false, message: error.message || 'Network error occurred' };
	        }
	    },
	    fetchProfile: async () => {
	        const token = localStorage.getItem('auth_token');
	        if (!token)
	            return;
	        try {
	            const result = await api.get('/users/profile');
	            if (result.success) {
	                authStore.update(state => ({
	                    ...state,
	                    user: result.data,
	                    isAuthenticated: true,
	                }));
	            }
	        }
	        catch (error) {
	            console.error('Failed to fetch profile:', error);
	        }
	    },
	    logout: async () => {
	        const token = localStorage.getItem('auth_token');
	        if (token) {
	            try {
	                await api.post('/auth/logout');
	            }
	            catch (error) {
	                console.error('Logout request failed:', error);
	            }
	        }
	        localStorage.removeItem('auth_token');
	        localStorage.removeItem('refresh_token');
	        authStore.set({
	            user: null,
	            token: null,
	            refreshToken: null,
	            isAuthenticated: false,
	            isLoading: false,
	        });
	    },
	    refreshToken: async () => {
	        const refreshToken = localStorage.getItem('refresh_token');
	        if (!refreshToken)
	            return false;
	        try {
	            const result = await api.post('/auth/refresh', { refresh_token: refreshToken });
	            if (result.success) {
	                const { access_token, refresh_token: newRefreshToken } = result.data;
	                localStorage.setItem('auth_token', access_token);
	                localStorage.setItem('refresh_token', newRefreshToken);
	                authStore.update(state => ({
	                    ...state,
	                    token: access_token,
	                    refreshToken: newRefreshToken,
	                    isAuthenticated: true,
	                }));
	                return true;
	            }
	        }
	        catch (error) {
	            console.error('Token refresh failed:', error);
	        }
	        return false;
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
	};
	// Initialize auth state on app load
	if (typeof window !== 'undefined') {
	    const token = localStorage.getItem('auth_token');
	    if (token) {
	        authStore.update(state => ({
	            ...state,
	            token,
	            isAuthenticated: true,
	        }));
	        authActions.fetchProfile();
	    }
	}

	const currentRoute = writable('overview');
	const routes = {
	    overview: {
	        path: '/',
	        component: null, // Will be handled in Dashboard component
	        title: 'Overview'
	    },
	    connections: {
	        path: '/connections',
	        component: null,
	        title: 'Service Connections'
	    },
	    dnp: {
	        path: '/dnp',
	        component: null,
	        title: 'DNP List'
	    },
	    enforcement: {
	        path: '/enforcement',
	        component: null,
	        title: 'Enforcement'
	    },
	    community: {
	        path: '/community',
	        component: null,
	        title: 'Community Lists'
	    },
	    profile: {
	        path: '/profile',
	        component: null,
	        title: 'Profile & Settings'
	    }
	};
	const router = {
	    navigate: (route) => {
	        currentRoute.set(route);
	        // Update URL without page reload
	        if (typeof window !== 'undefined') {
	            const routeConfig = routes[route];
	            if (routeConfig) {
	                window.history.pushState({}, routeConfig.title, routeConfig.path);
	                document.title = `${routeConfig.title} - No Drake in the House`;
	            }
	        }
	    },
	    init: () => {
	        if (typeof window !== 'undefined') {
	            // Handle browser back/forward buttons
	            window.addEventListener('popstate', () => {
	                const path = window.location.pathname;
	                const route = Object.keys(routes).find(key => routes[key].path === path) || 'overview';
	                currentRoute.set(route);
	            });
	            // Set initial route based on URL
	            const path = window.location.pathname;
	            const route = Object.keys(routes).find(key => routes[key].path === path) || 'overview';
	            currentRoute.set(route);
	        }
	    }
	};

	/* src/lib/components/LoginForm.svelte generated by Svelte v4.2.20 */
	const file$n = "src/lib/components/LoginForm.svelte";

	// (63:8) {#if !emailValid}
	function create_if_block_4$f(ctx) {
		let p;

		const block = {
			c: function create() {
				p = element("p");
				p.textContent = "Please enter a valid email address";
				attr_dev(p, "class", "mt-1 text-sm text-red-600");
				add_location(p, file$n, 73, 10, 2139);
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
			id: create_if_block_4$f.name,
			type: "if",
			source: "(63:8) {#if !emailValid}",
			ctx
		});

		return block;
	}

	// (88:8) {#if !passwordValid}
	function create_if_block_3$g(ctx) {
		let p;

		const block = {
			c: function create() {
				p = element("p");
				p.textContent = "Password must be at least 8 characters";
				attr_dev(p, "class", "mt-1 text-sm text-red-600");
				add_location(p, file$n, 98, 10, 3058);
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
			id: create_if_block_3$g.name,
			type: "if",
			source: "(88:8) {#if !passwordValid}",
			ctx
		});

		return block;
	}

	// (95:4) {#if showTotpInput}
	function create_if_block_2$h(ctx) {
		let div1;
		let label;
		let t1;
		let div0;
		let input;
		let t2;
		let p;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				div1 = element("div");
				label = element("label");
				label.textContent = "2FA Authentication Code";
				t1 = space();
				div0 = element("div");
				input = element("input");
				t2 = space();
				p = element("p");
				p.textContent = "Enter the 6-digit code from your authenticator app";
				attr_dev(label, "for", "login-totp");
				attr_dev(label, "class", "block text-sm font-medium text-gray-700");
				add_location(label, file$n, 106, 8, 3271);
				attr_dev(input, "id", "login-totp");
				attr_dev(input, "name", "totp");
				attr_dev(input, "type", "text");
				attr_dev(input, "autocomplete", "one-time-code");
				attr_dev(input, "maxlength", "6");
				attr_dev(input, "pattern", "[0-9]" + 6);
				attr_dev(input, "class", "appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm");
				attr_dev(input, "placeholder", "Enter 6-digit code");
				add_location(input, file$n, 110, 10, 3432);
				attr_dev(p, "class", "mt-1 text-sm text-gray-500");
				add_location(p, file$n, 121, 10, 3905);
				attr_dev(div0, "class", "mt-1");
				add_location(div0, file$n, 109, 8, 3403);
				add_location(div1, file$n, 105, 6, 3257);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div1, anchor);
				append_dev(div1, label);
				append_dev(div1, t1);
				append_dev(div1, div0);
				append_dev(div0, input);
				set_input_value(input, /*totpCode*/ ctx[6]);
				append_dev(div0, t2);
				append_dev(div0, p);

				if (!mounted) {
					dispose = listen_dev(input, "input", /*input_input_handler*/ ctx[13]);
					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (dirty & /*totpCode*/ 64 && input.value !== /*totpCode*/ ctx[6]) {
					set_input_value(input, /*totpCode*/ ctx[6]);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div1);
				}

				mounted = false;
				dispose();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_2$h.name,
			type: "if",
			source: "(95:4) {#if showTotpInput}",
			ctx
		});

		return block;
	}

	// (120:4) {#if error}
	function create_if_block_1$k(ctx) {
		let div3;
		let div2;
		let div0;
		let svg;
		let path;
		let t0;
		let div1;
		let p;
		let t1;

		const block = {
			c: function create() {
				div3 = element("div");
				div2 = element("div");
				div0 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				div1 = element("div");
				p = element("p");
				t1 = text(/*error*/ ctx[1]);
				attr_dev(path, "fill-rule", "evenodd");
				attr_dev(path, "d", "M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z");
				attr_dev(path, "clip-rule", "evenodd");
				add_location(path, file$n, 134, 14, 4315);
				attr_dev(svg, "class", "h-5 w-5 text-red-400");
				attr_dev(svg, "viewBox", "0 0 20 20");
				attr_dev(svg, "fill", "currentColor");
				add_location(svg, file$n, 133, 12, 4226);
				attr_dev(div0, "class", "flex-shrink-0");
				add_location(div0, file$n, 132, 10, 4186);
				attr_dev(p, "class", "text-sm text-red-800");
				add_location(p, file$n, 138, 12, 4661);
				attr_dev(div1, "class", "ml-3");
				add_location(div1, file$n, 137, 10, 4630);
				attr_dev(div2, "class", "flex");
				add_location(div2, file$n, 131, 8, 4157);
				attr_dev(div3, "class", "rounded-md bg-red-50 p-4");
				add_location(div3, file$n, 130, 6, 4110);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div3, anchor);
				append_dev(div3, div2);
				append_dev(div2, div0);
				append_dev(div0, svg);
				append_dev(svg, path);
				append_dev(div2, t0);
				append_dev(div2, div1);
				append_dev(div1, p);
				append_dev(p, t1);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*error*/ 2) set_data_dev(t1, /*error*/ ctx[1]);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div3);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_1$k.name,
			type: "if",
			source: "(120:4) {#if error}",
			ctx
		});

		return block;
	}

	// (148:8) {:else}
	function create_else_block$i(ctx) {
		let t;

		const block = {
			c: function create() {
				t = text("Sign in");
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
			id: create_else_block$i.name,
			type: "else",
			source: "(148:8) {:else}",
			ctx
		});

		return block;
	}

	// (142:8) {#if isLoading}
	function create_if_block$l(ctx) {
		let svg;
		let circle;
		let path;
		let t;

		const block = {
			c: function create() {
				svg = svg_element("svg");
				circle = svg_element("circle");
				path = svg_element("path");
				t = text("\n          Signing in...");
				attr_dev(circle, "class", "opacity-25");
				attr_dev(circle, "cx", "12");
				attr_dev(circle, "cy", "12");
				attr_dev(circle, "r", "10");
				attr_dev(circle, "stroke", "currentColor");
				attr_dev(circle, "stroke-width", "4");
				add_location(circle, file$n, 153, 12, 5347);
				attr_dev(path, "class", "opacity-75");
				attr_dev(path, "fill", "currentColor");
				attr_dev(path, "d", "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z");
				add_location(path, file$n, 154, 12, 5458);
				attr_dev(svg, "class", "animate-spin -ml-1 mr-3 h-5 w-5 text-white");
				attr_dev(svg, "xmlns", "http://www.w3.org/2000/svg");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				add_location(svg, file$n, 152, 10, 5211);
			},
			m: function mount(target, anchor) {
				insert_dev(target, svg, anchor);
				append_dev(svg, circle);
				append_dev(svg, path);
				insert_dev(target, t, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(svg);
					detach_dev(t);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block$l.name,
			type: "if",
			source: "(142:8) {#if isLoading}",
			ctx
		});

		return block;
	}

	function create_fragment$n(ctx) {
		let div7;
		let div0;
		let h2;
		let t1;
		let p;
		let t3;
		let form;
		let div2;
		let label0;
		let t5;
		let div1;
		let input0;
		let t6;
		let t7;
		let div4;
		let label1;
		let t9;
		let div3;
		let input1;
		let t10;
		let t11;
		let t12;
		let t13;
		let div5;
		let button0;
		let button0_disabled_value;
		let t14;
		let div6;
		let button1;
		let mounted;
		let dispose;
		let if_block0 = !/*emailValid*/ ctx[5] && create_if_block_4$f(ctx);
		let if_block1 = !/*passwordValid*/ ctx[4] && create_if_block_3$g(ctx);
		let if_block2 = /*showTotpInput*/ ctx[7] && create_if_block_2$h(ctx);
		let if_block3 = /*error*/ ctx[1] && create_if_block_1$k(ctx);

		function select_block_type(ctx, dirty) {
			if (/*isLoading*/ ctx[0]) return create_if_block$l;
			return create_else_block$i;
		}

		let current_block_type = select_block_type(ctx);
		let if_block4 = current_block_type(ctx);

		const block = {
			c: function create() {
				div7 = element("div");
				div0 = element("div");
				h2 = element("h2");
				h2.textContent = "Sign in to your account";
				t1 = space();
				p = element("p");
				p.textContent = "Access your music blocklist manager";
				t3 = space();
				form = element("form");
				div2 = element("div");
				label0 = element("label");
				label0.textContent = "Email address";
				t5 = space();
				div1 = element("div");
				input0 = element("input");
				t6 = space();
				if (if_block0) if_block0.c();
				t7 = space();
				div4 = element("div");
				label1 = element("label");
				label1.textContent = "Password";
				t9 = space();
				div3 = element("div");
				input1 = element("input");
				t10 = space();
				if (if_block1) if_block1.c();
				t11 = space();
				if (if_block2) if_block2.c();
				t12 = space();
				if (if_block3) if_block3.c();
				t13 = space();
				div5 = element("div");
				button0 = element("button");
				if_block4.c();
				t14 = space();
				div6 = element("div");
				button1 = element("button");
				button1.textContent = "Don't have an account? Create one";
				attr_dev(h2, "class", "text-center text-3xl font-extrabold text-gray-900");
				add_location(h2, file$n, 44, 4, 1086);
				attr_dev(p, "class", "mt-2 text-center text-sm text-gray-600");
				add_location(p, file$n, 47, 4, 1193);
				add_location(div0, file$n, 43, 2, 1076);
				attr_dev(label0, "for", "login-email");
				attr_dev(label0, "class", "block text-sm font-medium text-gray-700");
				add_location(label0, file$n, 55, 6, 1415);
				attr_dev(input0, "id", "login-email");
				attr_dev(input0, "name", "email");
				attr_dev(input0, "type", "email");
				attr_dev(input0, "autocomplete", "email");
				input0.required = true;
				attr_dev(input0, "class", "appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm");
				attr_dev(input0, "placeholder", "Enter your email");
				toggle_class(input0, "border-red-300", !/*emailValid*/ ctx[5]);
				toggle_class(input0, "focus:ring-red-500", !/*emailValid*/ ctx[5]);
				toggle_class(input0, "focus:border-red-500", !/*emailValid*/ ctx[5]);
				add_location(input0, file$n, 59, 8, 1559);
				attr_dev(div1, "class", "mt-1");
				add_location(div1, file$n, 58, 6, 1532);
				add_location(div2, file$n, 54, 4, 1403);
				attr_dev(label1, "for", "login-password");
				attr_dev(label1, "class", "block text-sm font-medium text-gray-700");
				add_location(label1, file$n, 80, 6, 2298);
				attr_dev(input1, "id", "login-password");
				attr_dev(input1, "name", "password");
				attr_dev(input1, "type", "password");
				attr_dev(input1, "autocomplete", "current-password");
				input1.required = true;
				attr_dev(input1, "class", "appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm");
				attr_dev(input1, "placeholder", "Enter your password");
				toggle_class(input1, "border-red-300", !/*passwordValid*/ ctx[4]);
				toggle_class(input1, "focus:ring-red-500", !/*passwordValid*/ ctx[4]);
				toggle_class(input1, "focus:border-red-500", !/*passwordValid*/ ctx[4]);
				add_location(input1, file$n, 84, 8, 2440);
				attr_dev(div3, "class", "mt-1");
				add_location(div3, file$n, 83, 6, 2413);
				add_location(div4, file$n, 79, 4, 2286);
				attr_dev(button0, "type", "submit");
				button0.disabled = button0_disabled_value = !/*formValid*/ ctx[8] || /*isLoading*/ ctx[0];
				attr_dev(button0, "class", "group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed");
				add_location(button0, file$n, 146, 6, 4804);
				add_location(div5, file$n, 145, 4, 4792);
				attr_dev(button1, "type", "button");
				attr_dev(button1, "class", "text-indigo-600 hover:text-indigo-500 text-sm font-medium");
				add_location(button1, file$n, 165, 6, 5812);
				attr_dev(div6, "class", "text-center");
				add_location(div6, file$n, 164, 4, 5780);
				attr_dev(form, "class", "space-y-4");
				add_location(form, file$n, 52, 2, 1309);
				attr_dev(div7, "class", "space-y-6");
				add_location(div7, file$n, 42, 0, 1050);
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				insert_dev(target, div7, anchor);
				append_dev(div7, div0);
				append_dev(div0, h2);
				append_dev(div0, t1);
				append_dev(div0, p);
				append_dev(div7, t3);
				append_dev(div7, form);
				append_dev(form, div2);
				append_dev(div2, label0);
				append_dev(div2, t5);
				append_dev(div2, div1);
				append_dev(div1, input0);
				set_input_value(input0, /*email*/ ctx[2]);
				append_dev(div1, t6);
				if (if_block0) if_block0.m(div1, null);
				append_dev(form, t7);
				append_dev(form, div4);
				append_dev(div4, label1);
				append_dev(div4, t9);
				append_dev(div4, div3);
				append_dev(div3, input1);
				set_input_value(input1, /*password*/ ctx[3]);
				append_dev(div3, t10);
				if (if_block1) if_block1.m(div3, null);
				append_dev(form, t11);
				if (if_block2) if_block2.m(form, null);
				append_dev(form, t12);
				if (if_block3) if_block3.m(form, null);
				append_dev(form, t13);
				append_dev(form, div5);
				append_dev(div5, button0);
				if_block4.m(button0, null);
				append_dev(form, t14);
				append_dev(form, div6);
				append_dev(div6, button1);

				if (!mounted) {
					dispose = [
						listen_dev(input0, "input", /*input0_input_handler*/ ctx[11]),
						listen_dev(input1, "input", /*input1_input_handler*/ ctx[12]),
						listen_dev(button1, "click", /*click_handler*/ ctx[14], false, false, false, false),
						listen_dev(form, "submit", prevent_default(/*handleSubmit*/ ctx[10]), false, true, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, [dirty]) {
				if (dirty & /*email*/ 4 && input0.value !== /*email*/ ctx[2]) {
					set_input_value(input0, /*email*/ ctx[2]);
				}

				if (dirty & /*emailValid*/ 32) {
					toggle_class(input0, "border-red-300", !/*emailValid*/ ctx[5]);
				}

				if (dirty & /*emailValid*/ 32) {
					toggle_class(input0, "focus:ring-red-500", !/*emailValid*/ ctx[5]);
				}

				if (dirty & /*emailValid*/ 32) {
					toggle_class(input0, "focus:border-red-500", !/*emailValid*/ ctx[5]);
				}

				if (!/*emailValid*/ ctx[5]) {
					if (if_block0) ; else {
						if_block0 = create_if_block_4$f(ctx);
						if_block0.c();
						if_block0.m(div1, null);
					}
				} else if (if_block0) {
					if_block0.d(1);
					if_block0 = null;
				}

				if (dirty & /*password*/ 8 && input1.value !== /*password*/ ctx[3]) {
					set_input_value(input1, /*password*/ ctx[3]);
				}

				if (dirty & /*passwordValid*/ 16) {
					toggle_class(input1, "border-red-300", !/*passwordValid*/ ctx[4]);
				}

				if (dirty & /*passwordValid*/ 16) {
					toggle_class(input1, "focus:ring-red-500", !/*passwordValid*/ ctx[4]);
				}

				if (dirty & /*passwordValid*/ 16) {
					toggle_class(input1, "focus:border-red-500", !/*passwordValid*/ ctx[4]);
				}

				if (!/*passwordValid*/ ctx[4]) {
					if (if_block1) ; else {
						if_block1 = create_if_block_3$g(ctx);
						if_block1.c();
						if_block1.m(div3, null);
					}
				} else if (if_block1) {
					if_block1.d(1);
					if_block1 = null;
				}

				if (/*showTotpInput*/ ctx[7]) {
					if (if_block2) {
						if_block2.p(ctx, dirty);
					} else {
						if_block2 = create_if_block_2$h(ctx);
						if_block2.c();
						if_block2.m(form, t12);
					}
				} else if (if_block2) {
					if_block2.d(1);
					if_block2 = null;
				}

				if (/*error*/ ctx[1]) {
					if (if_block3) {
						if_block3.p(ctx, dirty);
					} else {
						if_block3 = create_if_block_1$k(ctx);
						if_block3.c();
						if_block3.m(form, t13);
					}
				} else if (if_block3) {
					if_block3.d(1);
					if_block3 = null;
				}

				if (current_block_type !== (current_block_type = select_block_type(ctx))) {
					if_block4.d(1);
					if_block4 = current_block_type(ctx);

					if (if_block4) {
						if_block4.c();
						if_block4.m(button0, null);
					}
				}

				if (dirty & /*formValid, isLoading*/ 257 && button0_disabled_value !== (button0_disabled_value = !/*formValid*/ ctx[8] || /*isLoading*/ ctx[0])) {
					prop_dev(button0, "disabled", button0_disabled_value);
				}
			},
			i: noop,
			o: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div7);
				}

				if (if_block0) if_block0.d();
				if (if_block1) if_block1.d();
				if (if_block2) if_block2.d();
				if (if_block3) if_block3.d();
				if_block4.d();
				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_fragment$n.name,
			type: "component",
			source: "",
			ctx
		});

		return block;
	}

	function instance$n($$self, $$props, $$invalidate) {
		let emailValid;
		let passwordValid;
		let formValid;
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('LoginForm', slots, []);
		const dispatch = createEventDispatcher();
		let { isLoading = false } = $$props;
		let { error = '' } = $$props;
		let email = '';
		let password = '';
		let totpCode = '';
		let showTotpInput = false;

		function handleSubmit() {
			if (!formValid) return;

			dispatch('login', {
				email: email.trim(),
				password,
				totpCode: totpCode || undefined
			});
		}

		const writable_props = ['isLoading', 'error'];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<LoginForm> was created with unknown prop '${key}'`);
		});

		function input0_input_handler() {
			email = this.value;
			$$invalidate(2, email);
		}

		function input1_input_handler() {
			password = this.value;
			$$invalidate(3, password);
		}

		function input_input_handler() {
			totpCode = this.value;
			($$invalidate(6, totpCode), $$invalidate(1, error));
		}

		const click_handler = () => dispatch('switchMode');

		$$self.$$set = $$props => {
			if ('isLoading' in $$props) $$invalidate(0, isLoading = $$props.isLoading);
			if ('error' in $$props) $$invalidate(1, error = $$props.error);
		};

		$$self.$capture_state = () => ({
			createEventDispatcher,
			dispatch,
			isLoading,
			error,
			email,
			password,
			totpCode,
			showTotpInput,
			handleSubmit,
			formValid,
			passwordValid,
			emailValid
		});

		$$self.$inject_state = $$props => {
			if ('isLoading' in $$props) $$invalidate(0, isLoading = $$props.isLoading);
			if ('error' in $$props) $$invalidate(1, error = $$props.error);
			if ('email' in $$props) $$invalidate(2, email = $$props.email);
			if ('password' in $$props) $$invalidate(3, password = $$props.password);
			if ('totpCode' in $$props) $$invalidate(6, totpCode = $$props.totpCode);
			if ('showTotpInput' in $$props) $$invalidate(7, showTotpInput = $$props.showTotpInput);
			if ('formValid' in $$props) $$invalidate(8, formValid = $$props.formValid);
			if ('passwordValid' in $$props) $$invalidate(4, passwordValid = $$props.passwordValid);
			if ('emailValid' in $$props) $$invalidate(5, emailValid = $$props.emailValid);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		$$self.$$.update = () => {
			if ($$self.$$.dirty & /*email*/ 4) {
				// Email validation
				$$invalidate(5, emailValid = email.length === 0 || (/^[^\s@]+@[^\s@]+\.[^\s@]+$/).test(email));
			}

			if ($$self.$$.dirty & /*password*/ 8) {
				$$invalidate(4, passwordValid = password.length === 0 || password.length >= 8);
			}

			if ($$self.$$.dirty & /*emailValid, passwordValid, email, password*/ 60) {
				$$invalidate(8, formValid = emailValid && passwordValid && email.length > 0 && password.length > 0);
			}

			if ($$self.$$.dirty & /*error*/ 2) {
				// Reset TOTP input when error changes
				if (error && !error.toLowerCase().includes('2fa') && !error.toLowerCase().includes('totp')) {
					$$invalidate(7, showTotpInput = false);
					$$invalidate(6, totpCode = '');
				}
			}

			if ($$self.$$.dirty & /*error*/ 2) {
				// Show TOTP input if error indicates it's required
				if (error && (error.toLowerCase().includes('2fa') || error.toLowerCase().includes('totp'))) {
					$$invalidate(7, showTotpInput = true);
				}
			}
		};

		return [
			isLoading,
			error,
			email,
			password,
			passwordValid,
			emailValid,
			totpCode,
			showTotpInput,
			formValid,
			dispatch,
			handleSubmit,
			input0_input_handler,
			input1_input_handler,
			input_input_handler,
			click_handler
		];
	}

	class LoginForm extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance$n, create_fragment$n, safe_not_equal, { isLoading: 0, error: 1 });

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "LoginForm",
				options,
				id: create_fragment$n.name
			});
		}

		get isLoading() {
			throw new Error("<LoginForm>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}

		set isLoading(value) {
			throw new Error("<LoginForm>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}

		get error() {
			throw new Error("<LoginForm>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}

		set error(value) {
			throw new Error("<LoginForm>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}
	}

	/* src/lib/components/RegisterForm.svelte generated by Svelte v4.2.20 */
	const file$m = "src/lib/components/RegisterForm.svelte";

	function get_each_context$9(ctx, list, i) {
		const child_ctx = ctx.slice();
		child_ctx[24] = list[i];
		child_ctx[26] = i;
		return child_ctx;
	}

	// (76:8) {#if !emailValid}
	function create_if_block_5$d(ctx) {
		let p;

		const block = {
			c: function create() {
				p = element("p");
				p.textContent = "Please enter a valid email address";
				attr_dev(p, "class", "mt-1 text-sm text-red-600");
				add_location(p, file$m, 86, 10, 2739);
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
			id: create_if_block_5$d.name,
			type: "if",
			source: "(76:8) {#if !emailValid}",
			ctx
		});

		return block;
	}

	// (103:8) {#if password.length > 0}
	function create_if_block_4$e(ctx) {
		let div2;
		let div0;
		let span0;
		let t1;
		let span1;
		let t2;
		let span1_class_value;
		let t3;
		let div1;
		let t4;
		let div9;
		let p;
		let t6;
		let div8;
		let div3;
		let svg0;
		let path0;
		let t7;
		let span2;
		let t9;
		let div4;
		let svg1;
		let path1;
		let t10;
		let span3;
		let t12;
		let div5;
		let svg2;
		let path2;
		let t13;
		let span4;
		let t15;
		let div6;
		let svg3;
		let path3;
		let t16;
		let span5;
		let t18;
		let div7;
		let svg4;
		let path4;
		let t19;
		let span6;
		let each_value = ensure_array_like_dev(Array(5));
		let each_blocks = [];

		for (let i = 0; i < each_value.length; i += 1) {
			each_blocks[i] = create_each_block$9(get_each_context$9(ctx, each_value, i));
		}

		const block = {
			c: function create() {
				div2 = element("div");
				div0 = element("div");
				span0 = element("span");
				span0.textContent = "Password strength:";
				t1 = space();
				span1 = element("span");
				t2 = text(/*passwordStrengthText*/ ctx[17]);
				t3 = space();
				div1 = element("div");

				for (let i = 0; i < each_blocks.length; i += 1) {
					each_blocks[i].c();
				}

				t4 = space();
				div9 = element("div");
				p = element("p");
				p.textContent = "Password must contain:";
				t6 = space();
				div8 = element("div");
				div3 = element("div");
				svg0 = svg_element("svg");
				path0 = svg_element("path");
				t7 = space();
				span2 = element("span");
				span2.textContent = "At least 8 characters";
				t9 = space();
				div4 = element("div");
				svg1 = svg_element("svg");
				path1 = svg_element("path");
				t10 = space();
				span3 = element("span");
				span3.textContent = "One uppercase letter";
				t12 = space();
				div5 = element("div");
				svg2 = svg_element("svg");
				path2 = svg_element("path");
				t13 = space();
				span4 = element("span");
				span4.textContent = "One lowercase letter";
				t15 = space();
				div6 = element("div");
				svg3 = svg_element("svg");
				path3 = svg_element("path");
				t16 = space();
				span5 = element("span");
				span5.textContent = "One number";
				t18 = space();
				div7 = element("div");
				svg4 = svg_element("svg");
				path4 = svg_element("path");
				t19 = space();
				span6 = element("span");
				span6.textContent = "One special character";
				attr_dev(span0, "class", "text-sm text-gray-600");
				add_location(span0, file$m, 115, 14, 3886);
				attr_dev(span1, "class", span1_class_value = "text-sm font-medium " + /*passwordStrengthColor*/ ctx[16]);
				add_location(span1, file$m, 116, 14, 3962);
				attr_dev(div0, "class", "flex items-center justify-between");
				add_location(div0, file$m, 114, 12, 3824);
				attr_dev(div1, "class", "mt-1 flex space-x-1");
				add_location(div1, file$m, 120, 12, 4113);
				attr_dev(div2, "class", "mt-2");
				add_location(div2, file$m, 113, 10, 3793);
				attr_dev(p, "class", "text-xs text-gray-600");
				add_location(p, file$m, 140, 12, 5089);
				attr_dev(path0, "fill-rule", "evenodd");
				attr_dev(path0, "d", "M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z");
				attr_dev(path0, "clip-rule", "evenodd");
				add_location(path0, file$m, 144, 18, 5429);
				attr_dev(svg0, "class", "h-3 w-3 mr-1");
				attr_dev(svg0, "fill", "currentColor");
				attr_dev(svg0, "viewBox", "0 0 20 20");
				toggle_class(svg0, "text-green-500", /*passwordLength*/ ctx[10]);
				toggle_class(svg0, "text-gray-400", !/*passwordLength*/ ctx[10]);
				add_location(svg0, file$m, 143, 16, 5268);
				toggle_class(span2, "text-green-600", /*passwordLength*/ ctx[10]);
				toggle_class(span2, "text-gray-500", !/*passwordLength*/ ctx[10]);
				add_location(span2, file$m, 146, 16, 5636);
				attr_dev(div3, "class", "flex items-center");
				add_location(div3, file$m, 142, 14, 5220);
				attr_dev(path1, "fill-rule", "evenodd");
				attr_dev(path1, "d", "M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z");
				attr_dev(path1, "clip-rule", "evenodd");
				add_location(path1, file$m, 152, 18, 6033);
				attr_dev(svg1, "class", "h-3 w-3 mr-1");
				attr_dev(svg1, "fill", "currentColor");
				attr_dev(svg1, "viewBox", "0 0 20 20");
				toggle_class(svg1, "text-green-500", /*passwordUppercase*/ ctx[14]);
				toggle_class(svg1, "text-gray-400", !/*passwordUppercase*/ ctx[14]);
				add_location(svg1, file$m, 151, 16, 5866);
				toggle_class(span3, "text-green-600", /*passwordUppercase*/ ctx[14]);
				toggle_class(span3, "text-gray-500", !/*passwordUppercase*/ ctx[14]);
				add_location(span3, file$m, 154, 16, 6240);
				attr_dev(div4, "class", "flex items-center");
				add_location(div4, file$m, 150, 14, 5818);
				attr_dev(path2, "fill-rule", "evenodd");
				attr_dev(path2, "d", "M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z");
				attr_dev(path2, "clip-rule", "evenodd");
				add_location(path2, file$m, 160, 18, 6642);
				attr_dev(svg2, "class", "h-3 w-3 mr-1");
				attr_dev(svg2, "fill", "currentColor");
				attr_dev(svg2, "viewBox", "0 0 20 20");
				toggle_class(svg2, "text-green-500", /*passwordLowercase*/ ctx[13]);
				toggle_class(svg2, "text-gray-400", !/*passwordLowercase*/ ctx[13]);
				add_location(svg2, file$m, 159, 16, 6475);
				toggle_class(span4, "text-green-600", /*passwordLowercase*/ ctx[13]);
				toggle_class(span4, "text-gray-500", !/*passwordLowercase*/ ctx[13]);
				add_location(span4, file$m, 162, 16, 6849);
				attr_dev(div5, "class", "flex items-center");
				add_location(div5, file$m, 158, 14, 6427);
				attr_dev(path3, "fill-rule", "evenodd");
				attr_dev(path3, "d", "M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z");
				attr_dev(path3, "clip-rule", "evenodd");
				add_location(path3, file$m, 168, 18, 7245);
				attr_dev(svg3, "class", "h-3 w-3 mr-1");
				attr_dev(svg3, "fill", "currentColor");
				attr_dev(svg3, "viewBox", "0 0 20 20");
				toggle_class(svg3, "text-green-500", /*passwordNumber*/ ctx[12]);
				toggle_class(svg3, "text-gray-400", !/*passwordNumber*/ ctx[12]);
				add_location(svg3, file$m, 167, 16, 7084);
				toggle_class(span5, "text-green-600", /*passwordNumber*/ ctx[12]);
				toggle_class(span5, "text-gray-500", !/*passwordNumber*/ ctx[12]);
				add_location(span5, file$m, 170, 16, 7452);
				attr_dev(div6, "class", "flex items-center");
				add_location(div6, file$m, 166, 14, 7036);
				attr_dev(path4, "fill-rule", "evenodd");
				attr_dev(path4, "d", "M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z");
				attr_dev(path4, "clip-rule", "evenodd");
				add_location(path4, file$m, 176, 18, 7834);
				attr_dev(svg4, "class", "h-3 w-3 mr-1");
				attr_dev(svg4, "fill", "currentColor");
				attr_dev(svg4, "viewBox", "0 0 20 20");
				toggle_class(svg4, "text-green-500", /*passwordSpecial*/ ctx[11]);
				toggle_class(svg4, "text-gray-400", !/*passwordSpecial*/ ctx[11]);
				add_location(svg4, file$m, 175, 16, 7671);
				toggle_class(span6, "text-green-600", /*passwordSpecial*/ ctx[11]);
				toggle_class(span6, "text-gray-500", !/*passwordSpecial*/ ctx[11]);
				add_location(span6, file$m, 178, 16, 8041);
				attr_dev(div7, "class", "flex items-center");
				add_location(div7, file$m, 174, 14, 7623);
				attr_dev(div8, "class", "grid grid-cols-1 gap-1 text-xs");
				add_location(div8, file$m, 141, 12, 5161);
				attr_dev(div9, "class", "mt-2 space-y-1");
				add_location(div9, file$m, 139, 10, 5048);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div2, anchor);
				append_dev(div2, div0);
				append_dev(div0, span0);
				append_dev(div0, t1);
				append_dev(div0, span1);
				append_dev(span1, t2);
				append_dev(div2, t3);
				append_dev(div2, div1);

				for (let i = 0; i < each_blocks.length; i += 1) {
					if (each_blocks[i]) {
						each_blocks[i].m(div1, null);
					}
				}

				insert_dev(target, t4, anchor);
				insert_dev(target, div9, anchor);
				append_dev(div9, p);
				append_dev(div9, t6);
				append_dev(div9, div8);
				append_dev(div8, div3);
				append_dev(div3, svg0);
				append_dev(svg0, path0);
				append_dev(div3, t7);
				append_dev(div3, span2);
				append_dev(div8, t9);
				append_dev(div8, div4);
				append_dev(div4, svg1);
				append_dev(svg1, path1);
				append_dev(div4, t10);
				append_dev(div4, span3);
				append_dev(div8, t12);
				append_dev(div8, div5);
				append_dev(div5, svg2);
				append_dev(svg2, path2);
				append_dev(div5, t13);
				append_dev(div5, span4);
				append_dev(div8, t15);
				append_dev(div8, div6);
				append_dev(div6, svg3);
				append_dev(svg3, path3);
				append_dev(div6, t16);
				append_dev(div6, span5);
				append_dev(div8, t18);
				append_dev(div8, div7);
				append_dev(div7, svg4);
				append_dev(svg4, path4);
				append_dev(div7, t19);
				append_dev(div7, span6);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*passwordStrengthText*/ 131072) set_data_dev(t2, /*passwordStrengthText*/ ctx[17]);

				if (dirty & /*passwordStrengthColor*/ 65536 && span1_class_value !== (span1_class_value = "text-sm font-medium " + /*passwordStrengthColor*/ ctx[16])) {
					attr_dev(span1, "class", span1_class_value);
				}

				if (dirty & /*passwordStrength*/ 512) {
					each_value = ensure_array_like_dev(Array(5));
					let i;

					for (i = 0; i < each_value.length; i += 1) {
						const child_ctx = get_each_context$9(ctx, each_value, i);

						if (each_blocks[i]) {
							each_blocks[i].p(child_ctx, dirty);
						} else {
							each_blocks[i] = create_each_block$9(child_ctx);
							each_blocks[i].c();
							each_blocks[i].m(div1, null);
						}
					}

					for (; i < each_blocks.length; i += 1) {
						each_blocks[i].d(1);
					}

					each_blocks.length = each_value.length;
				}

				if (dirty & /*passwordLength*/ 1024) {
					toggle_class(svg0, "text-green-500", /*passwordLength*/ ctx[10]);
				}

				if (dirty & /*passwordLength*/ 1024) {
					toggle_class(svg0, "text-gray-400", !/*passwordLength*/ ctx[10]);
				}

				if (dirty & /*passwordLength*/ 1024) {
					toggle_class(span2, "text-green-600", /*passwordLength*/ ctx[10]);
				}

				if (dirty & /*passwordLength*/ 1024) {
					toggle_class(span2, "text-gray-500", !/*passwordLength*/ ctx[10]);
				}

				if (dirty & /*passwordUppercase*/ 16384) {
					toggle_class(svg1, "text-green-500", /*passwordUppercase*/ ctx[14]);
				}

				if (dirty & /*passwordUppercase*/ 16384) {
					toggle_class(svg1, "text-gray-400", !/*passwordUppercase*/ ctx[14]);
				}

				if (dirty & /*passwordUppercase*/ 16384) {
					toggle_class(span3, "text-green-600", /*passwordUppercase*/ ctx[14]);
				}

				if (dirty & /*passwordUppercase*/ 16384) {
					toggle_class(span3, "text-gray-500", !/*passwordUppercase*/ ctx[14]);
				}

				if (dirty & /*passwordLowercase*/ 8192) {
					toggle_class(svg2, "text-green-500", /*passwordLowercase*/ ctx[13]);
				}

				if (dirty & /*passwordLowercase*/ 8192) {
					toggle_class(svg2, "text-gray-400", !/*passwordLowercase*/ ctx[13]);
				}

				if (dirty & /*passwordLowercase*/ 8192) {
					toggle_class(span4, "text-green-600", /*passwordLowercase*/ ctx[13]);
				}

				if (dirty & /*passwordLowercase*/ 8192) {
					toggle_class(span4, "text-gray-500", !/*passwordLowercase*/ ctx[13]);
				}

				if (dirty & /*passwordNumber*/ 4096) {
					toggle_class(svg3, "text-green-500", /*passwordNumber*/ ctx[12]);
				}

				if (dirty & /*passwordNumber*/ 4096) {
					toggle_class(svg3, "text-gray-400", !/*passwordNumber*/ ctx[12]);
				}

				if (dirty & /*passwordNumber*/ 4096) {
					toggle_class(span5, "text-green-600", /*passwordNumber*/ ctx[12]);
				}

				if (dirty & /*passwordNumber*/ 4096) {
					toggle_class(span5, "text-gray-500", !/*passwordNumber*/ ctx[12]);
				}

				if (dirty & /*passwordSpecial*/ 2048) {
					toggle_class(svg4, "text-green-500", /*passwordSpecial*/ ctx[11]);
				}

				if (dirty & /*passwordSpecial*/ 2048) {
					toggle_class(svg4, "text-gray-400", !/*passwordSpecial*/ ctx[11]);
				}

				if (dirty & /*passwordSpecial*/ 2048) {
					toggle_class(span6, "text-green-600", /*passwordSpecial*/ ctx[11]);
				}

				if (dirty & /*passwordSpecial*/ 2048) {
					toggle_class(span6, "text-gray-500", !/*passwordSpecial*/ ctx[11]);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div2);
					detach_dev(t4);
					detach_dev(div9);
				}

				destroy_each(each_blocks, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_4$e.name,
			type: "if",
			source: "(103:8) {#if password.length > 0}",
			ctx
		});

		return block;
	}

	// (112:14) {#each Array(5) as _, i}
	function create_each_block$9(ctx) {
		let div;

		const block = {
			c: function create() {
				div = element("div");
				attr_dev(div, "class", "h-1 flex-1 rounded-full");
				toggle_class(div, "bg-red-200", /*passwordStrength*/ ctx[9] <= 2);
				toggle_class(div, "bg-yellow-200", /*passwordStrength*/ ctx[9] === 3);
				toggle_class(div, "bg-blue-200", /*passwordStrength*/ ctx[9] === 4);
				toggle_class(div, "bg-green-200", /*passwordStrength*/ ctx[9] === 5);
				toggle_class(div, "bg-red-500", /*passwordStrength*/ ctx[9] <= 2 && /*i*/ ctx[26] < /*passwordStrength*/ ctx[9]);
				toggle_class(div, "bg-yellow-500", /*passwordStrength*/ ctx[9] === 3 && /*i*/ ctx[26] < /*passwordStrength*/ ctx[9]);
				toggle_class(div, "bg-blue-500", /*passwordStrength*/ ctx[9] === 4 && /*i*/ ctx[26] < /*passwordStrength*/ ctx[9]);
				toggle_class(div, "bg-green-500", /*passwordStrength*/ ctx[9] === 5 && /*i*/ ctx[26] < /*passwordStrength*/ ctx[9]);
				toggle_class(div, "bg-gray-200", /*i*/ ctx[26] >= /*passwordStrength*/ ctx[9]);
				add_location(div, file$m, 122, 16, 4202);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*passwordStrength*/ 512) {
					toggle_class(div, "bg-red-200", /*passwordStrength*/ ctx[9] <= 2);
				}

				if (dirty & /*passwordStrength*/ 512) {
					toggle_class(div, "bg-yellow-200", /*passwordStrength*/ ctx[9] === 3);
				}

				if (dirty & /*passwordStrength*/ 512) {
					toggle_class(div, "bg-blue-200", /*passwordStrength*/ ctx[9] === 4);
				}

				if (dirty & /*passwordStrength*/ 512) {
					toggle_class(div, "bg-green-200", /*passwordStrength*/ ctx[9] === 5);
				}

				if (dirty & /*passwordStrength*/ 512) {
					toggle_class(div, "bg-red-500", /*passwordStrength*/ ctx[9] <= 2 && /*i*/ ctx[26] < /*passwordStrength*/ ctx[9]);
				}

				if (dirty & /*passwordStrength*/ 512) {
					toggle_class(div, "bg-yellow-500", /*passwordStrength*/ ctx[9] === 3 && /*i*/ ctx[26] < /*passwordStrength*/ ctx[9]);
				}

				if (dirty & /*passwordStrength*/ 512) {
					toggle_class(div, "bg-blue-500", /*passwordStrength*/ ctx[9] === 4 && /*i*/ ctx[26] < /*passwordStrength*/ ctx[9]);
				}

				if (dirty & /*passwordStrength*/ 512) {
					toggle_class(div, "bg-green-500", /*passwordStrength*/ ctx[9] === 5 && /*i*/ ctx[26] < /*passwordStrength*/ ctx[9]);
				}

				if (dirty & /*passwordStrength*/ 512) {
					toggle_class(div, "bg-gray-200", /*i*/ ctx[26] >= /*passwordStrength*/ ctx[9]);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_each_block$9.name,
			type: "each",
			source: "(112:14) {#each Array(5) as _, i}",
			ctx
		});

		return block;
	}

	// (198:8) {#if !passwordsMatch && confirmPassword.length > 0}
	function create_if_block_3$f(ctx) {
		let p;

		const block = {
			c: function create() {
				p = element("p");
				p.textContent = "Passwords do not match";
				attr_dev(p, "class", "mt-1 text-sm text-red-600");
				add_location(p, file$m, 208, 10, 9246);
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
			id: create_if_block_3$f.name,
			type: "if",
			source: "(198:8) {#if !passwordsMatch && confirmPassword.length > 0}",
			ctx
		});

		return block;
	}

	// (205:4) {#if success}
	function create_if_block_2$g(ctx) {
		let div3;
		let div2;
		let div0;
		let svg;
		let path;
		let t0;
		let div1;
		let p;
		let t1;

		const block = {
			c: function create() {
				div3 = element("div");
				div2 = element("div");
				div0 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				div1 = element("div");
				p = element("p");
				t1 = text(/*success*/ ctx[2]);
				attr_dev(path, "fill-rule", "evenodd");
				attr_dev(path, "d", "M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z");
				attr_dev(path, "clip-rule", "evenodd");
				add_location(path, file$m, 219, 14, 9611);
				attr_dev(svg, "class", "h-5 w-5 text-green-400");
				attr_dev(svg, "viewBox", "0 0 20 20");
				attr_dev(svg, "fill", "currentColor");
				add_location(svg, file$m, 218, 12, 9520);
				attr_dev(div0, "class", "flex-shrink-0");
				add_location(div0, file$m, 217, 10, 9480);
				attr_dev(p, "class", "text-sm text-green-800");
				add_location(p, file$m, 223, 12, 9875);
				attr_dev(div1, "class", "ml-3");
				add_location(div1, file$m, 222, 10, 9844);
				attr_dev(div2, "class", "flex");
				add_location(div2, file$m, 216, 8, 9451);
				attr_dev(div3, "class", "rounded-md bg-green-50 p-4");
				add_location(div3, file$m, 215, 6, 9402);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div3, anchor);
				append_dev(div3, div2);
				append_dev(div2, div0);
				append_dev(div0, svg);
				append_dev(svg, path);
				append_dev(div2, t0);
				append_dev(div2, div1);
				append_dev(div1, p);
				append_dev(p, t1);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*success*/ 4) set_data_dev(t1, /*success*/ ctx[2]);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div3);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_2$g.name,
			type: "if",
			source: "(205:4) {#if success}",
			ctx
		});

		return block;
	}

	// (221:4) {#if error}
	function create_if_block_1$j(ctx) {
		let div3;
		let div2;
		let div0;
		let svg;
		let path;
		let t0;
		let div1;
		let p;
		let t1;

		const block = {
			c: function create() {
				div3 = element("div");
				div2 = element("div");
				div0 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				div1 = element("div");
				p = element("p");
				t1 = text(/*error*/ ctx[1]);
				attr_dev(path, "fill-rule", "evenodd");
				attr_dev(path, "d", "M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z");
				attr_dev(path, "clip-rule", "evenodd");
				add_location(path, file$m, 235, 14, 10233);
				attr_dev(svg, "class", "h-5 w-5 text-red-400");
				attr_dev(svg, "viewBox", "0 0 20 20");
				attr_dev(svg, "fill", "currentColor");
				add_location(svg, file$m, 234, 12, 10144);
				attr_dev(div0, "class", "flex-shrink-0");
				add_location(div0, file$m, 233, 10, 10104);
				attr_dev(p, "class", "text-sm text-red-800");
				add_location(p, file$m, 239, 12, 10579);
				attr_dev(div1, "class", "ml-3");
				add_location(div1, file$m, 238, 10, 10548);
				attr_dev(div2, "class", "flex");
				add_location(div2, file$m, 232, 8, 10075);
				attr_dev(div3, "class", "rounded-md bg-red-50 p-4");
				add_location(div3, file$m, 231, 6, 10028);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div3, anchor);
				append_dev(div3, div2);
				append_dev(div2, div0);
				append_dev(div0, svg);
				append_dev(svg, path);
				append_dev(div2, t0);
				append_dev(div2, div1);
				append_dev(div1, p);
				append_dev(p, t1);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*error*/ 2) set_data_dev(t1, /*error*/ ctx[1]);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div3);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_1$j.name,
			type: "if",
			source: "(221:4) {#if error}",
			ctx
		});

		return block;
	}

	// (249:8) {:else}
	function create_else_block$h(ctx) {
		let t;

		const block = {
			c: function create() {
				t = text("Create account");
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
			id: create_else_block$h.name,
			type: "else",
			source: "(249:8) {:else}",
			ctx
		});

		return block;
	}

	// (243:8) {#if isLoading}
	function create_if_block$k(ctx) {
		let svg;
		let circle;
		let path;
		let t;

		const block = {
			c: function create() {
				svg = svg_element("svg");
				circle = svg_element("circle");
				path = svg_element("path");
				t = text("\n          Creating account...");
				attr_dev(circle, "class", "opacity-25");
				attr_dev(circle, "cx", "12");
				attr_dev(circle, "cy", "12");
				attr_dev(circle, "r", "10");
				attr_dev(circle, "stroke", "currentColor");
				attr_dev(circle, "stroke-width", "4");
				add_location(circle, file$m, 254, 12, 11265);
				attr_dev(path, "class", "opacity-75");
				attr_dev(path, "fill", "currentColor");
				attr_dev(path, "d", "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z");
				add_location(path, file$m, 255, 12, 11376);
				attr_dev(svg, "class", "animate-spin -ml-1 mr-3 h-5 w-5 text-white");
				attr_dev(svg, "xmlns", "http://www.w3.org/2000/svg");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				add_location(svg, file$m, 253, 10, 11129);
			},
			m: function mount(target, anchor) {
				insert_dev(target, svg, anchor);
				append_dev(svg, circle);
				append_dev(svg, path);
				insert_dev(target, t, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(svg);
					detach_dev(t);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block$k.name,
			type: "if",
			source: "(243:8) {#if isLoading}",
			ctx
		});

		return block;
	}

	function create_fragment$m(ctx) {
		let div9;
		let div0;
		let h2;
		let t1;
		let p;
		let t3;
		let form;
		let div2;
		let label0;
		let t5;
		let div1;
		let input0;
		let t6;
		let t7;
		let div4;
		let label1;
		let t9;
		let div3;
		let input1;
		let t10;
		let t11;
		let div6;
		let label2;
		let t13;
		let div5;
		let input2;
		let t14;
		let t15;
		let t16;
		let t17;
		let div7;
		let button0;
		let button0_disabled_value;
		let t18;
		let div8;
		let button1;
		let mounted;
		let dispose;
		let if_block0 = !/*emailValid*/ ctx[8] && create_if_block_5$d(ctx);
		let if_block1 = /*password*/ ctx[4].length > 0 && create_if_block_4$e(ctx);
		let if_block2 = !/*passwordsMatch*/ ctx[6] && /*confirmPassword*/ ctx[5].length > 0 && create_if_block_3$f(ctx);
		let if_block3 = /*success*/ ctx[2] && create_if_block_2$g(ctx);
		let if_block4 = /*error*/ ctx[1] && create_if_block_1$j(ctx);

		function select_block_type(ctx, dirty) {
			if (/*isLoading*/ ctx[0]) return create_if_block$k;
			return create_else_block$h;
		}

		let current_block_type = select_block_type(ctx);
		let if_block5 = current_block_type(ctx);

		const block = {
			c: function create() {
				div9 = element("div");
				div0 = element("div");
				h2 = element("h2");
				h2.textContent = "Create your account";
				t1 = space();
				p = element("p");
				p.textContent = "Join the music blocklist community";
				t3 = space();
				form = element("form");
				div2 = element("div");
				label0 = element("label");
				label0.textContent = "Email address";
				t5 = space();
				div1 = element("div");
				input0 = element("input");
				t6 = space();
				if (if_block0) if_block0.c();
				t7 = space();
				div4 = element("div");
				label1 = element("label");
				label1.textContent = "Password";
				t9 = space();
				div3 = element("div");
				input1 = element("input");
				t10 = space();
				if (if_block1) if_block1.c();
				t11 = space();
				div6 = element("div");
				label2 = element("label");
				label2.textContent = "Confirm Password";
				t13 = space();
				div5 = element("div");
				input2 = element("input");
				t14 = space();
				if (if_block2) if_block2.c();
				t15 = space();
				if (if_block3) if_block3.c();
				t16 = space();
				if (if_block4) if_block4.c();
				t17 = space();
				div7 = element("div");
				button0 = element("button");
				if_block5.c();
				t18 = space();
				div8 = element("div");
				button1 = element("button");
				button1.textContent = "Already have an account? Sign in";
				attr_dev(h2, "class", "text-center text-3xl font-extrabold text-gray-900");
				add_location(h2, file$m, 57, 4, 1685);
				attr_dev(p, "class", "mt-2 text-center text-sm text-gray-600");
				add_location(p, file$m, 60, 4, 1788);
				add_location(div0, file$m, 56, 2, 1675);
				attr_dev(label0, "for", "register-email");
				attr_dev(label0, "class", "block text-sm font-medium text-gray-700");
				add_location(label0, file$m, 68, 6, 2009);
				attr_dev(input0, "id", "register-email");
				attr_dev(input0, "name", "email");
				attr_dev(input0, "type", "email");
				attr_dev(input0, "autocomplete", "email");
				input0.required = true;
				attr_dev(input0, "class", "appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm");
				attr_dev(input0, "placeholder", "Enter your email");
				toggle_class(input0, "border-red-300", !/*emailValid*/ ctx[8]);
				toggle_class(input0, "focus:ring-red-500", !/*emailValid*/ ctx[8]);
				toggle_class(input0, "focus:border-red-500", !/*emailValid*/ ctx[8]);
				add_location(input0, file$m, 72, 8, 2156);
				attr_dev(div1, "class", "mt-1");
				add_location(div1, file$m, 71, 6, 2129);
				add_location(div2, file$m, 67, 4, 1997);
				attr_dev(label1, "for", "register-password");
				attr_dev(label1, "class", "block text-sm font-medium text-gray-700");
				add_location(label1, file$m, 93, 6, 2898);
				attr_dev(input1, "id", "register-password");
				attr_dev(input1, "name", "password");
				attr_dev(input1, "type", "password");
				attr_dev(input1, "autocomplete", "new-password");
				input1.required = true;
				attr_dev(input1, "class", "appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm");
				attr_dev(input1, "placeholder", "Create a strong password");
				toggle_class(input1, "border-red-300", !/*passwordValid*/ ctx[7] && /*password*/ ctx[4].length > 0);
				toggle_class(input1, "focus:ring-red-500", !/*passwordValid*/ ctx[7] && /*password*/ ctx[4].length > 0);
				toggle_class(input1, "focus:border-red-500", !/*passwordValid*/ ctx[7] && /*password*/ ctx[4].length > 0);
				add_location(input1, file$m, 97, 8, 3043);
				attr_dev(div3, "class", "mt-1");
				add_location(div3, file$m, 96, 6, 3016);
				add_location(div4, file$m, 92, 4, 2886);
				attr_dev(label2, "for", "confirm-password");
				attr_dev(label2, "class", "block text-sm font-medium text-gray-700");
				add_location(label2, file$m, 190, 6, 8338);
				attr_dev(input2, "id", "confirm-password");
				attr_dev(input2, "name", "confirmPassword");
				attr_dev(input2, "type", "password");
				attr_dev(input2, "autocomplete", "new-password");
				input2.required = true;
				attr_dev(input2, "class", "appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm");
				attr_dev(input2, "placeholder", "Confirm your password");
				toggle_class(input2, "border-red-300", !/*passwordsMatch*/ ctx[6] && /*confirmPassword*/ ctx[5].length > 0);
				toggle_class(input2, "focus:ring-red-500", !/*passwordsMatch*/ ctx[6] && /*confirmPassword*/ ctx[5].length > 0);
				toggle_class(input2, "focus:border-red-500", !/*passwordsMatch*/ ctx[6] && /*confirmPassword*/ ctx[5].length > 0);
				add_location(input2, file$m, 194, 8, 8490);
				attr_dev(div5, "class", "mt-1");
				add_location(div5, file$m, 193, 6, 8463);
				add_location(div6, file$m, 189, 4, 8326);
				attr_dev(button0, "type", "submit");
				button0.disabled = button0_disabled_value = !/*formValid*/ ctx[15] || /*isLoading*/ ctx[0];
				attr_dev(button0, "class", "group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed");
				add_location(button0, file$m, 247, 6, 10722);
				add_location(div7, file$m, 246, 4, 10710);
				attr_dev(button1, "type", "button");
				attr_dev(button1, "class", "text-indigo-600 hover:text-indigo-500 text-sm font-medium");
				add_location(button1, file$m, 266, 6, 11740);
				attr_dev(div8, "class", "text-center");
				add_location(div8, file$m, 265, 4, 11708);
				attr_dev(form, "class", "space-y-4");
				add_location(form, file$m, 65, 2, 1903);
				attr_dev(div9, "class", "space-y-6");
				add_location(div9, file$m, 55, 0, 1649);
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				insert_dev(target, div9, anchor);
				append_dev(div9, div0);
				append_dev(div0, h2);
				append_dev(div0, t1);
				append_dev(div0, p);
				append_dev(div9, t3);
				append_dev(div9, form);
				append_dev(form, div2);
				append_dev(div2, label0);
				append_dev(div2, t5);
				append_dev(div2, div1);
				append_dev(div1, input0);
				set_input_value(input0, /*email*/ ctx[3]);
				append_dev(div1, t6);
				if (if_block0) if_block0.m(div1, null);
				append_dev(form, t7);
				append_dev(form, div4);
				append_dev(div4, label1);
				append_dev(div4, t9);
				append_dev(div4, div3);
				append_dev(div3, input1);
				set_input_value(input1, /*password*/ ctx[4]);
				append_dev(div3, t10);
				if (if_block1) if_block1.m(div3, null);
				append_dev(form, t11);
				append_dev(form, div6);
				append_dev(div6, label2);
				append_dev(div6, t13);
				append_dev(div6, div5);
				append_dev(div5, input2);
				set_input_value(input2, /*confirmPassword*/ ctx[5]);
				append_dev(div5, t14);
				if (if_block2) if_block2.m(div5, null);
				append_dev(form, t15);
				if (if_block3) if_block3.m(form, null);
				append_dev(form, t16);
				if (if_block4) if_block4.m(form, null);
				append_dev(form, t17);
				append_dev(form, div7);
				append_dev(div7, button0);
				if_block5.m(button0, null);
				append_dev(form, t18);
				append_dev(form, div8);
				append_dev(div8, button1);

				if (!mounted) {
					dispose = [
						listen_dev(input0, "input", /*input0_input_handler*/ ctx[20]),
						listen_dev(input1, "input", /*input1_input_handler*/ ctx[21]),
						listen_dev(input2, "input", /*input2_input_handler*/ ctx[22]),
						listen_dev(button1, "click", /*click_handler*/ ctx[23], false, false, false, false),
						listen_dev(form, "submit", prevent_default(/*handleSubmit*/ ctx[19]), false, true, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, [dirty]) {
				if (dirty & /*email*/ 8 && input0.value !== /*email*/ ctx[3]) {
					set_input_value(input0, /*email*/ ctx[3]);
				}

				if (dirty & /*emailValid*/ 256) {
					toggle_class(input0, "border-red-300", !/*emailValid*/ ctx[8]);
				}

				if (dirty & /*emailValid*/ 256) {
					toggle_class(input0, "focus:ring-red-500", !/*emailValid*/ ctx[8]);
				}

				if (dirty & /*emailValid*/ 256) {
					toggle_class(input0, "focus:border-red-500", !/*emailValid*/ ctx[8]);
				}

				if (!/*emailValid*/ ctx[8]) {
					if (if_block0) ; else {
						if_block0 = create_if_block_5$d(ctx);
						if_block0.c();
						if_block0.m(div1, null);
					}
				} else if (if_block0) {
					if_block0.d(1);
					if_block0 = null;
				}

				if (dirty & /*password*/ 16 && input1.value !== /*password*/ ctx[4]) {
					set_input_value(input1, /*password*/ ctx[4]);
				}

				if (dirty & /*passwordValid, password*/ 144) {
					toggle_class(input1, "border-red-300", !/*passwordValid*/ ctx[7] && /*password*/ ctx[4].length > 0);
				}

				if (dirty & /*passwordValid, password*/ 144) {
					toggle_class(input1, "focus:ring-red-500", !/*passwordValid*/ ctx[7] && /*password*/ ctx[4].length > 0);
				}

				if (dirty & /*passwordValid, password*/ 144) {
					toggle_class(input1, "focus:border-red-500", !/*passwordValid*/ ctx[7] && /*password*/ ctx[4].length > 0);
				}

				if (/*password*/ ctx[4].length > 0) {
					if (if_block1) {
						if_block1.p(ctx, dirty);
					} else {
						if_block1 = create_if_block_4$e(ctx);
						if_block1.c();
						if_block1.m(div3, null);
					}
				} else if (if_block1) {
					if_block1.d(1);
					if_block1 = null;
				}

				if (dirty & /*confirmPassword*/ 32 && input2.value !== /*confirmPassword*/ ctx[5]) {
					set_input_value(input2, /*confirmPassword*/ ctx[5]);
				}

				if (dirty & /*passwordsMatch, confirmPassword*/ 96) {
					toggle_class(input2, "border-red-300", !/*passwordsMatch*/ ctx[6] && /*confirmPassword*/ ctx[5].length > 0);
				}

				if (dirty & /*passwordsMatch, confirmPassword*/ 96) {
					toggle_class(input2, "focus:ring-red-500", !/*passwordsMatch*/ ctx[6] && /*confirmPassword*/ ctx[5].length > 0);
				}

				if (dirty & /*passwordsMatch, confirmPassword*/ 96) {
					toggle_class(input2, "focus:border-red-500", !/*passwordsMatch*/ ctx[6] && /*confirmPassword*/ ctx[5].length > 0);
				}

				if (!/*passwordsMatch*/ ctx[6] && /*confirmPassword*/ ctx[5].length > 0) {
					if (if_block2) ; else {
						if_block2 = create_if_block_3$f(ctx);
						if_block2.c();
						if_block2.m(div5, null);
					}
				} else if (if_block2) {
					if_block2.d(1);
					if_block2 = null;
				}

				if (/*success*/ ctx[2]) {
					if (if_block3) {
						if_block3.p(ctx, dirty);
					} else {
						if_block3 = create_if_block_2$g(ctx);
						if_block3.c();
						if_block3.m(form, t16);
					}
				} else if (if_block3) {
					if_block3.d(1);
					if_block3 = null;
				}

				if (/*error*/ ctx[1]) {
					if (if_block4) {
						if_block4.p(ctx, dirty);
					} else {
						if_block4 = create_if_block_1$j(ctx);
						if_block4.c();
						if_block4.m(form, t17);
					}
				} else if (if_block4) {
					if_block4.d(1);
					if_block4 = null;
				}

				if (current_block_type !== (current_block_type = select_block_type(ctx))) {
					if_block5.d(1);
					if_block5 = current_block_type(ctx);

					if (if_block5) {
						if_block5.c();
						if_block5.m(button0, null);
					}
				}

				if (dirty & /*formValid, isLoading*/ 32769 && button0_disabled_value !== (button0_disabled_value = !/*formValid*/ ctx[15] || /*isLoading*/ ctx[0])) {
					prop_dev(button0, "disabled", button0_disabled_value);
				}
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
				if (if_block4) if_block4.d();
				if_block5.d();
				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_fragment$m.name,
			type: "component",
			source: "",
			ctx
		});

		return block;
	}

	function instance$m($$self, $$props, $$invalidate) {
		let passwordLength;
		let passwordUppercase;
		let passwordLowercase;
		let passwordNumber;
		let passwordSpecial;
		let passwordStrength;
		let passwordStrengthText;
		let passwordStrengthColor;
		let emailValid;
		let passwordValid;
		let passwordsMatch;
		let formValid;
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('RegisterForm', slots, []);
		const dispatch = createEventDispatcher();
		let { isLoading = false } = $$props;
		let { error = '' } = $$props;
		let { success = '' } = $$props;
		let email = '';
		let password = '';
		let confirmPassword = '';

		function handleSubmit() {
			if (!formValid) return;
			dispatch('register', { email: email.trim(), password });
		}

		const writable_props = ['isLoading', 'error', 'success'];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<RegisterForm> was created with unknown prop '${key}'`);
		});

		function input0_input_handler() {
			email = this.value;
			$$invalidate(3, email);
		}

		function input1_input_handler() {
			password = this.value;
			$$invalidate(4, password);
		}

		function input2_input_handler() {
			confirmPassword = this.value;
			$$invalidate(5, confirmPassword);
		}

		const click_handler = () => dispatch('switchMode');

		$$self.$$set = $$props => {
			if ('isLoading' in $$props) $$invalidate(0, isLoading = $$props.isLoading);
			if ('error' in $$props) $$invalidate(1, error = $$props.error);
			if ('success' in $$props) $$invalidate(2, success = $$props.success);
		};

		$$self.$capture_state = () => ({
			createEventDispatcher,
			dispatch,
			isLoading,
			error,
			success,
			email,
			password,
			confirmPassword,
			handleSubmit,
			formValid,
			passwordsMatch,
			passwordValid,
			emailValid,
			passwordStrength,
			passwordLength,
			passwordStrengthColor,
			passwordStrengthText,
			passwordSpecial,
			passwordNumber,
			passwordLowercase,
			passwordUppercase
		});

		$$self.$inject_state = $$props => {
			if ('isLoading' in $$props) $$invalidate(0, isLoading = $$props.isLoading);
			if ('error' in $$props) $$invalidate(1, error = $$props.error);
			if ('success' in $$props) $$invalidate(2, success = $$props.success);
			if ('email' in $$props) $$invalidate(3, email = $$props.email);
			if ('password' in $$props) $$invalidate(4, password = $$props.password);
			if ('confirmPassword' in $$props) $$invalidate(5, confirmPassword = $$props.confirmPassword);
			if ('formValid' in $$props) $$invalidate(15, formValid = $$props.formValid);
			if ('passwordsMatch' in $$props) $$invalidate(6, passwordsMatch = $$props.passwordsMatch);
			if ('passwordValid' in $$props) $$invalidate(7, passwordValid = $$props.passwordValid);
			if ('emailValid' in $$props) $$invalidate(8, emailValid = $$props.emailValid);
			if ('passwordStrength' in $$props) $$invalidate(9, passwordStrength = $$props.passwordStrength);
			if ('passwordLength' in $$props) $$invalidate(10, passwordLength = $$props.passwordLength);
			if ('passwordStrengthColor' in $$props) $$invalidate(16, passwordStrengthColor = $$props.passwordStrengthColor);
			if ('passwordStrengthText' in $$props) $$invalidate(17, passwordStrengthText = $$props.passwordStrengthText);
			if ('passwordSpecial' in $$props) $$invalidate(11, passwordSpecial = $$props.passwordSpecial);
			if ('passwordNumber' in $$props) $$invalidate(12, passwordNumber = $$props.passwordNumber);
			if ('passwordLowercase' in $$props) $$invalidate(13, passwordLowercase = $$props.passwordLowercase);
			if ('passwordUppercase' in $$props) $$invalidate(14, passwordUppercase = $$props.passwordUppercase);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		$$self.$$.update = () => {
			if ($$self.$$.dirty & /*password*/ 16) {
				// Password strength requirements
				$$invalidate(10, passwordLength = password.length >= 8);
			}

			if ($$self.$$.dirty & /*password*/ 16) {
				$$invalidate(14, passwordUppercase = (/[A-Z]/).test(password));
			}

			if ($$self.$$.dirty & /*password*/ 16) {
				$$invalidate(13, passwordLowercase = (/[a-z]/).test(password));
			}

			if ($$self.$$.dirty & /*password*/ 16) {
				$$invalidate(12, passwordNumber = (/\d/).test(password));
			}

			if ($$self.$$.dirty & /*password*/ 16) {
				$$invalidate(11, passwordSpecial = (/[!@#$%^&*(),.?":{}|<>]/).test(password));
			}

			if ($$self.$$.dirty & /*passwordLength, passwordUppercase, passwordLowercase, passwordNumber, passwordSpecial*/ 31744) {
				$$invalidate(9, passwordStrength = [
					passwordLength,
					passwordUppercase,
					passwordLowercase,
					passwordNumber,
					passwordSpecial
				].filter(Boolean).length);
			}

			if ($$self.$$.dirty & /*passwordStrength*/ 512) {
				$$invalidate(17, passwordStrengthText = passwordStrength === 0
				? ''
				: passwordStrength <= 2
					? 'Weak'
					: passwordStrength <= 3
						? 'Fair'
						: passwordStrength <= 4 ? 'Good' : 'Strong');
			}

			if ($$self.$$.dirty & /*passwordStrength*/ 512) {
				$$invalidate(16, passwordStrengthColor = passwordStrength === 0
				? ''
				: passwordStrength <= 2
					? 'text-red-600'
					: passwordStrength <= 3
						? 'text-yellow-600'
						: passwordStrength <= 4
							? 'text-blue-600'
							: 'text-green-600');
			}

			if ($$self.$$.dirty & /*email*/ 8) {
				// Validation
				$$invalidate(8, emailValid = email.length === 0 || (/^[^\s@]+@[^\s@]+\.[^\s@]+$/).test(email));
			}

			if ($$self.$$.dirty & /*password, passwordLength, passwordStrength*/ 1552) {
				$$invalidate(7, passwordValid = password.length === 0 || passwordLength && passwordStrength >= 3);
			}

			if ($$self.$$.dirty & /*confirmPassword, password*/ 48) {
				$$invalidate(6, passwordsMatch = confirmPassword.length === 0 || password === confirmPassword);
			}

			if ($$self.$$.dirty & /*emailValid, passwordValid, passwordsMatch, email, password, confirmPassword*/ 504) {
				$$invalidate(15, formValid = emailValid && passwordValid && passwordsMatch && email.length > 0 && password.length > 0 && confirmPassword.length > 0);
			}
		};

		return [
			isLoading,
			error,
			success,
			email,
			password,
			confirmPassword,
			passwordsMatch,
			passwordValid,
			emailValid,
			passwordStrength,
			passwordLength,
			passwordSpecial,
			passwordNumber,
			passwordLowercase,
			passwordUppercase,
			formValid,
			passwordStrengthColor,
			passwordStrengthText,
			dispatch,
			handleSubmit,
			input0_input_handler,
			input1_input_handler,
			input2_input_handler,
			click_handler
		];
	}

	class RegisterForm extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance$m, create_fragment$m, safe_not_equal, { isLoading: 0, error: 1, success: 2 });

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "RegisterForm",
				options,
				id: create_fragment$m.name
			});
		}

		get isLoading() {
			throw new Error("<RegisterForm>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}

		set isLoading(value) {
			throw new Error("<RegisterForm>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}

		get error() {
			throw new Error("<RegisterForm>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}

		set error(value) {
			throw new Error("<RegisterForm>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}

		get success() {
			throw new Error("<RegisterForm>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}

		set success(value) {
			throw new Error("<RegisterForm>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}
	}

	/* src/lib/components/TwoFactorSetup.svelte generated by Svelte v4.2.20 */
	const file$l = "src/lib/components/TwoFactorSetup.svelte";

	// (166:23) 
	function create_if_block_7$9(ctx) {
		let div6;
		let div0;
		let svg0;
		let path0;
		let t0;
		let div1;
		let h3;
		let t2;
		let p0;
		let t4;
		let div5;
		let div4;
		let div2;
		let svg1;
		let path1;
		let t5;
		let div3;
		let p1;
		let strong;
		let t7;
		let t8;
		let button;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				div6 = element("div");
				div0 = element("div");
				svg0 = svg_element("svg");
				path0 = svg_element("path");
				t0 = space();
				div1 = element("div");
				h3 = element("h3");
				h3.textContent = "Two-Factor Authentication Enabled!";
				t2 = space();
				p0 = element("p");
				p0.textContent = "Your account is now protected with 2FA. You'll need to enter a code from your authenticator app each time you sign in.";
				t4 = space();
				div5 = element("div");
				div4 = element("div");
				div2 = element("div");
				svg1 = svg_element("svg");
				path1 = svg_element("path");
				t5 = space();
				div3 = element("div");
				p1 = element("p");
				strong = element("strong");
				strong.textContent = "Important:";
				t7 = text(" Save your recovery codes in a safe place. You'll need them if you lose access to your authenticator app.");
				t8 = space();
				button = element("button");
				button.textContent = "Continue to Dashboard";
				attr_dev(path0, "stroke-linecap", "round");
				attr_dev(path0, "stroke-linejoin", "round");
				attr_dev(path0, "stroke-width", "2");
				attr_dev(path0, "d", "M5 13l4 4L19 7");
				add_location(path0, file$l, 179, 10, 6476);
				attr_dev(svg0, "class", "h-6 w-6 text-green-600");
				attr_dev(svg0, "fill", "none");
				attr_dev(svg0, "viewBox", "0 0 24 24");
				attr_dev(svg0, "stroke", "currentColor");
				add_location(svg0, file$l, 178, 8, 6375);
				attr_dev(div0, "class", "mx-auto flex items-center justify-center h-12 w-12 rounded-full bg-green-100");
				add_location(div0, file$l, 177, 6, 6276);
				attr_dev(h3, "class", "text-lg font-medium text-gray-900");
				add_location(h3, file$l, 184, 8, 6623);
				attr_dev(p0, "class", "mt-2 text-sm text-gray-600");
				add_location(p0, file$l, 187, 8, 6737);
				add_location(div1, file$l, 183, 6, 6609);
				attr_dev(path1, "fill-rule", "evenodd");
				attr_dev(path1, "d", "M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z");
				attr_dev(path1, "clip-rule", "evenodd");
				add_location(path1, file$l, 196, 14, 7149);
				attr_dev(svg1, "class", "h-5 w-5 text-yellow-400");
				attr_dev(svg1, "viewBox", "0 0 20 20");
				attr_dev(svg1, "fill", "currentColor");
				add_location(svg1, file$l, 195, 12, 7057);
				attr_dev(div2, "class", "flex-shrink-0");
				add_location(div2, file$l, 194, 10, 7017);
				add_location(strong, file$l, 201, 14, 7539);
				attr_dev(p1, "class", "text-sm text-yellow-800");
				add_location(p1, file$l, 200, 12, 7489);
				attr_dev(div3, "class", "ml-3");
				add_location(div3, file$l, 199, 10, 7458);
				attr_dev(div4, "class", "flex");
				add_location(div4, file$l, 193, 8, 6988);
				attr_dev(div5, "class", "bg-yellow-50 rounded-lg p-4");
				add_location(div5, file$l, 192, 6, 6938);
				attr_dev(button, "type", "button");
				attr_dev(button, "class", "w-full py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500");
				add_location(button, file$l, 207, 6, 7741);
				attr_dev(div6, "class", "text-center space-y-4");
				add_location(div6, file$l, 176, 4, 6234);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div6, anchor);
				append_dev(div6, div0);
				append_dev(div0, svg0);
				append_dev(svg0, path0);
				append_dev(div6, t0);
				append_dev(div6, div1);
				append_dev(div1, h3);
				append_dev(div1, t2);
				append_dev(div1, p0);
				append_dev(div6, t4);
				append_dev(div6, div5);
				append_dev(div5, div4);
				append_dev(div4, div2);
				append_dev(div2, svg1);
				append_dev(svg1, path1);
				append_dev(div4, t5);
				append_dev(div4, div3);
				append_dev(div3, p1);
				append_dev(p1, strong);
				append_dev(p1, t7);
				append_dev(div6, t8);
				append_dev(div6, button);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*click_handler_2*/ ctx[15], false, false, false, false);
					mounted = true;
				}
			},
			p: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div6);
				}

				mounted = false;
				dispose();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_7$9.name,
			type: "if",
			source: "(166:23) ",
			ctx
		});

		return block;
	}

	// (96:23) 
	function create_if_block_3$e(ctx) {
		let div3;
		let div1;
		let h3;
		let t1;
		let p;
		let t3;
		let div0;
		let input;
		let t4;
		let t5;
		let t6;
		let div2;
		let button0;
		let t8;
		let button1;
		let button1_disabled_value;
		let mounted;
		let dispose;
		let if_block0 = /*verificationCode*/ ctx[4].length > 0 && !/*codeValid*/ ctx[6] && create_if_block_6$c(ctx);
		let if_block1 = /*error*/ ctx[3] && create_if_block_5$c(ctx);

		function select_block_type_2(ctx, dirty) {
			if (/*isLoading*/ ctx[2]) return create_if_block_4$d;
			return create_else_block_1$9;
		}

		let current_block_type = select_block_type_2(ctx);
		let if_block2 = current_block_type(ctx);

		const block = {
			c: function create() {
				div3 = element("div");
				div1 = element("div");
				h3 = element("h3");
				h3.textContent = "Step 2: Verify Setup";
				t1 = space();
				p = element("p");
				p.textContent = "Enter the 6-digit code from your authenticator app to complete setup";
				t3 = space();
				div0 = element("div");
				input = element("input");
				t4 = space();
				if (if_block0) if_block0.c();
				t5 = space();
				if (if_block1) if_block1.c();
				t6 = space();
				div2 = element("div");
				button0 = element("button");
				button0.textContent = "Back";
				t8 = space();
				button1 = element("button");
				if_block2.c();
				attr_dev(h3, "class", "text-lg font-medium text-gray-900 mb-4");
				add_location(h3, file$l, 108, 8, 3164);
				attr_dev(p, "class", "text-sm text-gray-600 mb-4");
				add_location(p, file$l, 112, 8, 3278);
				attr_dev(input, "type", "text");
				attr_dev(input, "maxlength", "6");
				attr_dev(input, "pattern", "[0-9]" + 6);
				attr_dev(input, "placeholder", "000000");
				attr_dev(input, "class", "block w-full text-center text-2xl font-mono px-3 py-3 border border-gray-300 rounded-md placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500");
				toggle_class(input, "border-red-300", /*verificationCode*/ ctx[4].length > 0 && !/*codeValid*/ ctx[6]);
				add_location(input, file$l, 117, 10, 3467);
				attr_dev(div0, "class", "max-w-xs mx-auto");
				add_location(div0, file$l, 116, 8, 3426);
				attr_dev(div1, "class", "bg-gray-50 rounded-lg p-6 text-center");
				add_location(div1, file$l, 107, 6, 3104);
				attr_dev(button0, "type", "button");
				attr_dev(button0, "class", "flex-1 py-2 px-4 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500");
				add_location(button0, file$l, 148, 8, 4842);
				attr_dev(button1, "type", "button");
				button1.disabled = button1_disabled_value = !/*codeValid*/ ctx[6] || /*isLoading*/ ctx[2];
				attr_dev(button1, "class", "flex-1 py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed");
				add_location(button1, file$l, 155, 8, 5174);
				attr_dev(div2, "class", "flex space-x-3");
				add_location(div2, file$l, 147, 6, 4805);
				attr_dev(div3, "class", "space-y-4");
				add_location(div3, file$l, 106, 4, 3074);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div3, anchor);
				append_dev(div3, div1);
				append_dev(div1, h3);
				append_dev(div1, t1);
				append_dev(div1, p);
				append_dev(div1, t3);
				append_dev(div1, div0);
				append_dev(div0, input);
				set_input_value(input, /*verificationCode*/ ctx[4]);
				append_dev(div0, t4);
				if (if_block0) if_block0.m(div0, null);
				append_dev(div3, t5);
				if (if_block1) if_block1.m(div3, null);
				append_dev(div3, t6);
				append_dev(div3, div2);
				append_dev(div2, button0);
				append_dev(div2, t8);
				append_dev(div2, button1);
				if_block2.m(button1, null);

				if (!mounted) {
					dispose = [
						listen_dev(input, "input", /*input_input_handler*/ ctx[13]),
						listen_dev(button0, "click", /*click_handler_1*/ ctx[14], false, false, false, false),
						listen_dev(button1, "click", /*handleVerify*/ ctx[8], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (dirty & /*verificationCode*/ 16 && input.value !== /*verificationCode*/ ctx[4]) {
					set_input_value(input, /*verificationCode*/ ctx[4]);
				}

				if (dirty & /*verificationCode, codeValid*/ 80) {
					toggle_class(input, "border-red-300", /*verificationCode*/ ctx[4].length > 0 && !/*codeValid*/ ctx[6]);
				}

				if (/*verificationCode*/ ctx[4].length > 0 && !/*codeValid*/ ctx[6]) {
					if (if_block0) ; else {
						if_block0 = create_if_block_6$c(ctx);
						if_block0.c();
						if_block0.m(div0, null);
					}
				} else if (if_block0) {
					if_block0.d(1);
					if_block0 = null;
				}

				if (/*error*/ ctx[3]) {
					if (if_block1) {
						if_block1.p(ctx, dirty);
					} else {
						if_block1 = create_if_block_5$c(ctx);
						if_block1.c();
						if_block1.m(div3, t6);
					}
				} else if (if_block1) {
					if_block1.d(1);
					if_block1 = null;
				}

				if (current_block_type !== (current_block_type = select_block_type_2(ctx))) {
					if_block2.d(1);
					if_block2 = current_block_type(ctx);

					if (if_block2) {
						if_block2.c();
						if_block2.m(button1, null);
					}
				}

				if (dirty & /*codeValid, isLoading*/ 68 && button1_disabled_value !== (button1_disabled_value = !/*codeValid*/ ctx[6] || /*isLoading*/ ctx[2])) {
					prop_dev(button1, "disabled", button1_disabled_value);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div3);
				}

				if (if_block0) if_block0.d();
				if (if_block1) if_block1.d();
				if_block2.d();
				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_3$e.name,
			type: "if",
			source: "(96:23) ",
			ctx
		});

		return block;
	}

	// (41:2) {#if step === 1}
	function create_if_block_1$i(ctx) {
		let div3;
		let div0;
		let h3;
		let t1;
		let t2;
		let p;
		let t4;
		let div2;
		let h4;
		let t6;
		let div1;
		let code;
		let t7;
		let t8;
		let button0;
		let t10;
		let button1;
		let mounted;
		let dispose;

		function select_block_type_1(ctx, dirty) {
			if (/*qrCodeUrl*/ ctx[0]) return create_if_block_2$f;
			return create_else_block$g;
		}

		let current_block_type = select_block_type_1(ctx);
		let if_block = current_block_type(ctx);

		const block = {
			c: function create() {
				div3 = element("div");
				div0 = element("div");
				h3 = element("h3");
				h3.textContent = "Step 1: Scan QR Code";
				t1 = space();
				if_block.c();
				t2 = space();
				p = element("p");
				p.textContent = "Scan this QR code with your authenticator app (Google Authenticator, Authy, etc.)";
				t4 = space();
				div2 = element("div");
				h4 = element("h4");
				h4.textContent = "Can't scan? Enter manually:";
				t6 = space();
				div1 = element("div");
				code = element("code");
				t7 = text(/*secret*/ ctx[1]);
				t8 = space();
				button0 = element("button");
				button0.textContent = "Copy";
				t10 = space();
				button1 = element("button");
				button1.textContent = "I've added the account to my app";
				attr_dev(h3, "class", "text-lg font-medium text-gray-900 mb-4");
				add_location(h3, file$l, 53, 8, 1226);
				attr_dev(p, "class", "mt-4 text-sm text-gray-600");
				add_location(p, file$l, 71, 8, 1828);
				attr_dev(div0, "class", "bg-gray-50 rounded-lg p-6 text-center");
				add_location(div0, file$l, 52, 6, 1166);
				attr_dev(h4, "class", "text-sm font-medium text-blue-900 mb-2");
				add_location(h4, file$l, 78, 8, 2075);
				attr_dev(code, "class", "flex-1 bg-white px-3 py-2 rounded border text-sm font-mono");
				add_location(code, file$l, 82, 10, 2239);
				attr_dev(button0, "type", "button");
				attr_dev(button0, "class", "px-3 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500");
				add_location(button0, file$l, 85, 10, 2362);
				attr_dev(div1, "class", "flex items-center space-x-2");
				add_location(div1, file$l, 81, 8, 2187);
				attr_dev(div2, "class", "bg-blue-50 rounded-lg p-4");
				add_location(div2, file$l, 77, 6, 2027);
				attr_dev(button1, "type", "button");
				attr_dev(button1, "class", "w-full py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500");
				add_location(button1, file$l, 95, 6, 2653);
				attr_dev(div3, "class", "space-y-4");
				add_location(div3, file$l, 51, 4, 1136);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div3, anchor);
				append_dev(div3, div0);
				append_dev(div0, h3);
				append_dev(div0, t1);
				if_block.m(div0, null);
				append_dev(div0, t2);
				append_dev(div0, p);
				append_dev(div3, t4);
				append_dev(div3, div2);
				append_dev(div2, h4);
				append_dev(div2, t6);
				append_dev(div2, div1);
				append_dev(div1, code);
				append_dev(code, t7);
				append_dev(div1, t8);
				append_dev(div1, button0);
				append_dev(div3, t10);
				append_dev(div3, button1);

				if (!mounted) {
					dispose = [
						listen_dev(button0, "click", /*copySecret*/ ctx[10], false, false, false, false),
						listen_dev(button1, "click", /*click_handler*/ ctx[12], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (current_block_type === (current_block_type = select_block_type_1(ctx)) && if_block) {
					if_block.p(ctx, dirty);
				} else {
					if_block.d(1);
					if_block = current_block_type(ctx);

					if (if_block) {
						if_block.c();
						if_block.m(div0, t2);
					}
				}

				if (dirty & /*secret*/ 2) set_data_dev(t7, /*secret*/ ctx[1]);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div3);
				}

				if_block.d();
				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_1$i.name,
			type: "if",
			source: "(41:2) {#if step === 1}",
			ctx
		});

		return block;
	}

	// (118:10) {#if verificationCode.length > 0 && !codeValid}
	function create_if_block_6$c(ctx) {
		let p;

		const block = {
			c: function create() {
				p = element("p");
				p.textContent = "Please enter a 6-digit code";
				attr_dev(p, "class", "mt-1 text-sm text-red-600");
				add_location(p, file$l, 127, 12, 3984);
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
			id: create_if_block_6$c.name,
			type: "if",
			source: "(118:10) {#if verificationCode.length > 0 && !codeValid}",
			ctx
		});

		return block;
	}

	// (124:6) {#if error}
	function create_if_block_5$c(ctx) {
		let div3;
		let div2;
		let div0;
		let svg;
		let path;
		let t0;
		let div1;
		let p;
		let t1;

		const block = {
			c: function create() {
				div3 = element("div");
				div2 = element("div");
				div0 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				div1 = element("div");
				p = element("p");
				t1 = text(/*error*/ ctx[3]);
				attr_dev(path, "fill-rule", "evenodd");
				attr_dev(path, "d", "M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z");
				attr_dev(path, "clip-rule", "evenodd");
				add_location(path, file$l, 137, 16, 4337);
				attr_dev(svg, "class", "h-5 w-5 text-red-400");
				attr_dev(svg, "viewBox", "0 0 20 20");
				attr_dev(svg, "fill", "currentColor");
				add_location(svg, file$l, 136, 14, 4246);
				attr_dev(div0, "class", "flex-shrink-0");
				add_location(div0, file$l, 135, 12, 4204);
				attr_dev(p, "class", "text-sm text-red-800");
				add_location(p, file$l, 141, 14, 4691);
				attr_dev(div1, "class", "ml-3");
				add_location(div1, file$l, 140, 12, 4658);
				attr_dev(div2, "class", "flex");
				add_location(div2, file$l, 134, 10, 4173);
				attr_dev(div3, "class", "rounded-md bg-red-50 p-4");
				add_location(div3, file$l, 133, 8, 4124);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div3, anchor);
				append_dev(div3, div2);
				append_dev(div2, div0);
				append_dev(div0, svg);
				append_dev(svg, path);
				append_dev(div2, t0);
				append_dev(div2, div1);
				append_dev(div1, p);
				append_dev(p, t1);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*error*/ 8) set_data_dev(t1, /*error*/ ctx[3]);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div3);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_5$c.name,
			type: "if",
			source: "(124:6) {#if error}",
			ctx
		});

		return block;
	}

	// (159:10) {:else}
	function create_else_block_1$9(ctx) {
		let t;

		const block = {
			c: function create() {
				t = text("Verify & Enable");
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
			id: create_else_block_1$9.name,
			type: "else",
			source: "(159:10) {:else}",
			ctx
		});

		return block;
	}

	// (153:10) {#if isLoading}
	function create_if_block_4$d(ctx) {
		let svg;
		let circle;
		let path;
		let t;

		const block = {
			c: function create() {
				svg = svg_element("svg");
				circle = svg_element("circle");
				path = svg_element("path");
				t = text("\n            Verifying...");
				attr_dev(circle, "class", "opacity-25");
				attr_dev(circle, "cx", "12");
				attr_dev(circle, "cy", "12");
				attr_dev(circle, "r", "10");
				attr_dev(circle, "stroke", "currentColor");
				attr_dev(circle, "stroke-width", "4");
				add_location(circle, file$l, 163, 14, 5747);
				attr_dev(path, "class", "opacity-75");
				attr_dev(path, "fill", "currentColor");
				attr_dev(path, "d", "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z");
				add_location(path, file$l, 164, 14, 5860);
				attr_dev(svg, "class", "animate-spin -ml-1 mr-2 h-4 w-4 text-white inline");
				attr_dev(svg, "xmlns", "http://www.w3.org/2000/svg");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				add_location(svg, file$l, 162, 12, 5602);
			},
			m: function mount(target, anchor) {
				insert_dev(target, svg, anchor);
				append_dev(svg, circle);
				append_dev(svg, path);
				insert_dev(target, t, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(svg);
					detach_dev(t);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_4$d.name,
			type: "if",
			source: "(153:10) {#if isLoading}",
			ctx
		});

		return block;
	}

	// (57:8) {:else}
	function create_else_block$g(ctx) {
		let div1;
		let div0;

		const block = {
			c: function create() {
				div1 = element("div");
				div0 = element("div");
				attr_dev(div0, "class", "animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600");
				add_location(div0, file$l, 67, 12, 1697);
				attr_dev(div1, "class", "w-48 h-48 mx-auto bg-gray-200 rounded-lg flex items-center justify-center");
				add_location(div1, file$l, 66, 10, 1597);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div1, anchor);
				append_dev(div1, div0);
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
			id: create_else_block$g.name,
			type: "else",
			source: "(57:8) {:else}",
			ctx
		});

		return block;
	}

	// (49:8) {#if qrCodeUrl}
	function create_if_block_2$f(ctx) {
		let div;
		let img;
		let img_src_value;

		const block = {
			c: function create() {
				div = element("div");
				img = element("img");
				if (!src_url_equal(img.src, img_src_value = /*qrCodeUrl*/ ctx[0])) attr_dev(img, "src", img_src_value);
				attr_dev(img, "alt", "2FA QR Code");
				attr_dev(img, "class", "w-48 h-48 mx-auto");
				add_location(img, file$l, 59, 12, 1429);
				attr_dev(div, "class", "bg-white p-4 rounded-lg inline-block");
				add_location(div, file$l, 58, 10, 1366);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, img);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*qrCodeUrl*/ 1 && !src_url_equal(img.src, img_src_value = /*qrCodeUrl*/ ctx[0])) {
					attr_dev(img, "src", img_src_value);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_2$f.name,
			type: "if",
			source: "(49:8) {#if qrCodeUrl}",
			ctx
		});

		return block;
	}

	// (210:2) {#if step !== 3}
	function create_if_block$j(ctx) {
		let div;
		let button;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				div = element("div");
				button = element("button");
				button.textContent = "Skip for now";
				attr_dev(button, "type", "button");
				attr_dev(button, "class", "text-sm text-gray-500 hover:text-gray-700");
				add_location(button, file$l, 220, 6, 8215);
				attr_dev(div, "class", "text-center");
				add_location(div, file$l, 219, 4, 8183);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, button);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*handleCancel*/ ctx[9], false, false, false, false);
					mounted = true;
				}
			},
			p: noop,
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
			id: create_if_block$j.name,
			type: "if",
			source: "(210:2) {#if step !== 3}",
			ctx
		});

		return block;
	}

	function create_fragment$l(ctx) {
		let div1;
		let div0;
		let h2;
		let t1;
		let p;
		let t3;
		let t4;

		function select_block_type(ctx, dirty) {
			if (/*step*/ ctx[5] === 1) return create_if_block_1$i;
			if (/*step*/ ctx[5] === 2) return create_if_block_3$e;
			if (/*step*/ ctx[5] === 3) return create_if_block_7$9;
		}

		let current_block_type = select_block_type(ctx);
		let if_block0 = current_block_type && current_block_type(ctx);
		let if_block1 = /*step*/ ctx[5] !== 3 && create_if_block$j(ctx);

		const block = {
			c: function create() {
				div1 = element("div");
				div0 = element("div");
				h2 = element("h2");
				h2.textContent = "Set up Two-Factor Authentication";
				t1 = space();
				p = element("p");
				p.textContent = "Add an extra layer of security to your account";
				t3 = space();
				if (if_block0) if_block0.c();
				t4 = space();
				if (if_block1) if_block1.c();
				attr_dev(h2, "class", "text-2xl font-bold text-gray-900");
				add_location(h2, file$l, 41, 4, 871);
				attr_dev(p, "class", "mt-2 text-sm text-gray-600");
				add_location(p, file$l, 44, 4, 970);
				attr_dev(div0, "class", "text-center");
				add_location(div0, file$l, 40, 2, 841);
				attr_dev(div1, "class", "max-w-md mx-auto space-y-6");
				add_location(div1, file$l, 39, 0, 798);
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				insert_dev(target, div1, anchor);
				append_dev(div1, div0);
				append_dev(div0, h2);
				append_dev(div0, t1);
				append_dev(div0, p);
				append_dev(div1, t3);
				if (if_block0) if_block0.m(div1, null);
				append_dev(div1, t4);
				if (if_block1) if_block1.m(div1, null);
			},
			p: function update(ctx, [dirty]) {
				if (current_block_type === (current_block_type = select_block_type(ctx)) && if_block0) {
					if_block0.p(ctx, dirty);
				} else {
					if (if_block0) if_block0.d(1);
					if_block0 = current_block_type && current_block_type(ctx);

					if (if_block0) {
						if_block0.c();
						if_block0.m(div1, t4);
					}
				}

				if (/*step*/ ctx[5] !== 3) {
					if (if_block1) {
						if_block1.p(ctx, dirty);
					} else {
						if_block1 = create_if_block$j(ctx);
						if_block1.c();
						if_block1.m(div1, null);
					}
				} else if (if_block1) {
					if_block1.d(1);
					if_block1 = null;
				}
			},
			i: noop,
			o: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div1);
				}

				if (if_block0) {
					if_block0.d();
				}

				if (if_block1) if_block1.d();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_fragment$l.name,
			type: "component",
			source: "",
			ctx
		});

		return block;
	}

	function instance$l($$self, $$props, $$invalidate) {
		let codeValid;
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('TwoFactorSetup', slots, []);
		const dispatch = createEventDispatcher();
		let { qrCodeUrl = '' } = $$props;
		let { secret = '' } = $$props;
		let { isLoading = false } = $$props;
		let { error = '' } = $$props;
		let verificationCode = '';
		let step = 1; // 1: Show QR, 2: Verify code, 3: Success

		function handleVerify() {
			if (!codeValid) return;
			dispatch('verify', { code: verificationCode });
		}

		function handleCancel() {
			dispatch('cancel');
		}

		function copySecret() {
			navigator.clipboard.writeText(secret).then(() => {
				
			}); // Could add a toast notification here
		}

		function showSuccess() {
			$$invalidate(5, step = 3);
		}

		const writable_props = ['qrCodeUrl', 'secret', 'isLoading', 'error'];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<TwoFactorSetup> was created with unknown prop '${key}'`);
		});

		const click_handler = () => $$invalidate(5, step = 2);

		function input_input_handler() {
			verificationCode = this.value;
			$$invalidate(4, verificationCode);
		}

		const click_handler_1 = () => $$invalidate(5, step = 1);
		const click_handler_2 = () => dispatch('complete');

		$$self.$$set = $$props => {
			if ('qrCodeUrl' in $$props) $$invalidate(0, qrCodeUrl = $$props.qrCodeUrl);
			if ('secret' in $$props) $$invalidate(1, secret = $$props.secret);
			if ('isLoading' in $$props) $$invalidate(2, isLoading = $$props.isLoading);
			if ('error' in $$props) $$invalidate(3, error = $$props.error);
		};

		$$self.$capture_state = () => ({
			createEventDispatcher,
			dispatch,
			qrCodeUrl,
			secret,
			isLoading,
			error,
			verificationCode,
			step,
			handleVerify,
			handleCancel,
			copySecret,
			showSuccess,
			codeValid
		});

		$$self.$inject_state = $$props => {
			if ('qrCodeUrl' in $$props) $$invalidate(0, qrCodeUrl = $$props.qrCodeUrl);
			if ('secret' in $$props) $$invalidate(1, secret = $$props.secret);
			if ('isLoading' in $$props) $$invalidate(2, isLoading = $$props.isLoading);
			if ('error' in $$props) $$invalidate(3, error = $$props.error);
			if ('verificationCode' in $$props) $$invalidate(4, verificationCode = $$props.verificationCode);
			if ('step' in $$props) $$invalidate(5, step = $$props.step);
			if ('codeValid' in $$props) $$invalidate(6, codeValid = $$props.codeValid);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		$$self.$$.update = () => {
			if ($$self.$$.dirty & /*verificationCode*/ 16) {
				$$invalidate(6, codeValid = verificationCode.length === 6 && (/^\d{6}$/).test(verificationCode));
			}
		};

		return [
			qrCodeUrl,
			secret,
			isLoading,
			error,
			verificationCode,
			step,
			codeValid,
			dispatch,
			handleVerify,
			handleCancel,
			copySecret,
			showSuccess,
			click_handler,
			input_input_handler,
			click_handler_1,
			click_handler_2
		];
	}

	class TwoFactorSetup extends SvelteComponentDev {
		constructor(options) {
			super(options);

			init(this, options, instance$l, create_fragment$l, safe_not_equal, {
				qrCodeUrl: 0,
				secret: 1,
				isLoading: 2,
				error: 3,
				showSuccess: 11
			});

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "TwoFactorSetup",
				options,
				id: create_fragment$l.name
			});
		}

		get qrCodeUrl() {
			throw new Error("<TwoFactorSetup>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}

		set qrCodeUrl(value) {
			throw new Error("<TwoFactorSetup>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}

		get secret() {
			throw new Error("<TwoFactorSetup>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}

		set secret(value) {
			throw new Error("<TwoFactorSetup>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}

		get isLoading() {
			throw new Error("<TwoFactorSetup>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}

		set isLoading(value) {
			throw new Error("<TwoFactorSetup>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}

		get error() {
			throw new Error("<TwoFactorSetup>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}

		set error(value) {
			throw new Error("<TwoFactorSetup>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}

		get showSuccess() {
			return this.$$.ctx[11];
		}

		set showSuccess(value) {
			throw new Error("<TwoFactorSetup>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}
	}

	/* src/lib/components/TwoFactorVerification.svelte generated by Svelte v4.2.20 */
	const file$k = "src/lib/components/TwoFactorVerification.svelte";

	// (38:4) {#if email}
	function create_if_block_4$c(ctx) {
		let p;
		let t0;
		let span;
		let t1;

		const block = {
			c: function create() {
				p = element("p");
				t0 = text("Signing in as ");
				span = element("span");
				t1 = text(/*email*/ ctx[2]);
				attr_dev(span, "class", "font-medium");
				add_location(span, file$k, 47, 22, 1339);
				attr_dev(p, "class", "text-sm text-gray-500");
				add_location(p, file$k, 46, 6, 1283);
			},
			m: function mount(target, anchor) {
				insert_dev(target, p, anchor);
				append_dev(p, t0);
				append_dev(p, span);
				append_dev(span, t1);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*email*/ 4) set_data_dev(t1, /*email*/ ctx[2]);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(p);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_4$c.name,
			type: "if",
			source: "(38:4) {#if email}",
			ctx
		});

		return block;
	}

	// (64:8) {#if verificationCode.length > 0 && !codeValid}
	function create_if_block_3$d(ctx) {
		let p;

		const block = {
			c: function create() {
				p = element("p");
				p.textContent = "Please enter a 6-digit code";
				attr_dev(p, "class", "mt-1 text-sm text-red-600 text-center");
				add_location(p, file$k, 72, 10, 2276);
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
			id: create_if_block_3$d.name,
			type: "if",
			source: "(64:8) {#if verificationCode.length > 0 && !codeValid}",
			ctx
		});

		return block;
	}

	// (71:4) {#if error}
	function create_if_block_2$e(ctx) {
		let div3;
		let div2;
		let div0;
		let svg;
		let path;
		let t0;
		let div1;
		let p;
		let t1;

		const block = {
			c: function create() {
				div3 = element("div");
				div2 = element("div");
				div0 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				div1 = element("div");
				p = element("p");
				t1 = text(/*error*/ ctx[1]);
				attr_dev(path, "fill-rule", "evenodd");
				attr_dev(path, "d", "M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z");
				attr_dev(path, "clip-rule", "evenodd");
				add_location(path, file$k, 83, 14, 2650);
				attr_dev(svg, "class", "h-5 w-5 text-red-400");
				attr_dev(svg, "viewBox", "0 0 20 20");
				attr_dev(svg, "fill", "currentColor");
				add_location(svg, file$k, 82, 12, 2561);
				attr_dev(div0, "class", "flex-shrink-0");
				add_location(div0, file$k, 81, 10, 2521);
				attr_dev(p, "class", "text-sm text-red-800");
				add_location(p, file$k, 87, 12, 2996);
				attr_dev(div1, "class", "ml-3");
				add_location(div1, file$k, 86, 10, 2965);
				attr_dev(div2, "class", "flex");
				add_location(div2, file$k, 80, 8, 2492);
				attr_dev(div3, "class", "rounded-md bg-red-50 p-4");
				add_location(div3, file$k, 79, 6, 2445);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div3, anchor);
				append_dev(div3, div2);
				append_dev(div2, div0);
				append_dev(div0, svg);
				append_dev(svg, path);
				append_dev(div2, t0);
				append_dev(div2, div1);
				append_dev(div1, p);
				append_dev(p, t1);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*error*/ 2) set_data_dev(t1, /*error*/ ctx[1]);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div3);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_2$e.name,
			type: "if",
			source: "(71:4) {#if error}",
			ctx
		});

		return block;
	}

	// (87:4) {#if isLoading}
	function create_if_block_1$h(ctx) {
		let div1;
		let div0;
		let svg;
		let circle;
		let path;
		let t;

		const block = {
			c: function create() {
				div1 = element("div");
				div0 = element("div");
				svg = svg_element("svg");
				circle = svg_element("circle");
				path = svg_element("path");
				t = text("\n          Verifying code...");
				attr_dev(circle, "class", "opacity-25");
				attr_dev(circle, "cx", "12");
				attr_dev(circle, "cy", "12");
				attr_dev(circle, "r", "10");
				attr_dev(circle, "stroke", "currentColor");
				attr_dev(circle, "stroke-width", "4");
				add_location(circle, file$k, 98, 12, 3405);
				attr_dev(path, "class", "opacity-75");
				attr_dev(path, "fill", "currentColor");
				attr_dev(path, "d", "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z");
				add_location(path, file$k, 99, 12, 3516);
				attr_dev(svg, "class", "animate-spin -ml-1 mr-3 h-5 w-5 text-indigo-600");
				attr_dev(svg, "xmlns", "http://www.w3.org/2000/svg");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				add_location(svg, file$k, 97, 10, 3264);
				attr_dev(div0, "class", "inline-flex items-center px-4 py-2 text-sm text-gray-600");
				add_location(div0, file$k, 96, 8, 3183);
				attr_dev(div1, "class", "text-center");
				add_location(div1, file$k, 95, 6, 3149);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div1, anchor);
				append_dev(div1, div0);
				append_dev(div0, svg);
				append_dev(svg, circle);
				append_dev(svg, path);
				append_dev(div0, t);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div1);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_1$h.name,
			type: "if",
			source: "(87:4) {#if isLoading}",
			ctx
		});

		return block;
	}

	// (100:4) {#if codeValid && !isLoading}
	function create_if_block$i(ctx) {
		let button;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				button = element("button");
				button.textContent = "Verify Code";
				attr_dev(button, "type", "button");
				attr_dev(button, "class", "w-full py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500");
				add_location(button, file$k, 108, 6, 3866);
			},
			m: function mount(target, anchor) {
				insert_dev(target, button, anchor);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*handleSubmit*/ ctx[6], false, false, false, false);
					mounted = true;
				}
			},
			p: noop,
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
			id: create_if_block$i.name,
			type: "if",
			source: "(100:4) {#if codeValid && !isLoading}",
			ctx
		});

		return block;
	}

	function create_fragment$k(ctx) {
		let div9;
		let div1;
		let div0;
		let svg;
		let path;
		let t0;
		let h2;
		let t2;
		let p0;
		let t4;
		let t5;
		let div7;
		let div3;
		let label;
		let t7;
		let div2;
		let input;
		let t8;
		let t9;
		let t10;
		let t11;
		let t12;
		let div6;
		let p1;
		let t14;
		let div5;
		let button0;
		let t16;
		let div4;
		let button1;
		let t18;
		let div8;
		let h4;
		let t20;
		let ul;
		let li0;
		let t22;
		let li1;
		let t24;
		let li2;
		let mounted;
		let dispose;
		let if_block0 = /*email*/ ctx[2] && create_if_block_4$c(ctx);
		let if_block1 = /*verificationCode*/ ctx[3].length > 0 && !/*codeValid*/ ctx[4] && create_if_block_3$d(ctx);
		let if_block2 = /*error*/ ctx[1] && create_if_block_2$e(ctx);
		let if_block3 = /*isLoading*/ ctx[0] && create_if_block_1$h(ctx);
		let if_block4 = /*codeValid*/ ctx[4] && !/*isLoading*/ ctx[0] && create_if_block$i(ctx);

		const block = {
			c: function create() {
				div9 = element("div");
				div1 = element("div");
				div0 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				h2 = element("h2");
				h2.textContent = "Two-Factor Authentication";
				t2 = space();
				p0 = element("p");
				p0.textContent = "Enter the 6-digit code from your authenticator app";
				t4 = space();
				if (if_block0) if_block0.c();
				t5 = space();
				div7 = element("div");
				div3 = element("div");
				label = element("label");
				label.textContent = "Authentication Code";
				t7 = space();
				div2 = element("div");
				input = element("input");
				t8 = space();
				if (if_block1) if_block1.c();
				t9 = space();
				if (if_block2) if_block2.c();
				t10 = space();
				if (if_block3) if_block3.c();
				t11 = space();
				if (if_block4) if_block4.c();
				t12 = space();
				div6 = element("div");
				p1 = element("p");
				p1.textContent = "The code will automatically verify when you enter all 6 digits";
				t14 = space();
				div5 = element("div");
				button0 = element("button");
				button0.textContent = "Didn't receive a code? Try again";
				t16 = space();
				div4 = element("div");
				button1 = element("button");
				button1.textContent = "← Back to login";
				t18 = space();
				div8 = element("div");
				h4 = element("h4");
				h4.textContent = "Having trouble?";
				t20 = space();
				ul = element("ul");
				li0 = element("li");
				li0.textContent = "• Make sure your device's time is correct";
				t22 = space();
				li1 = element("li");
				li1.textContent = "• Try generating a new code in your authenticator app";
				t24 = space();
				li2 = element("li");
				li2.textContent = "• Check that you're using the right account in your app";
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z");
				add_location(path, file$k, 35, 8, 848);
				attr_dev(svg, "class", "h-6 w-6 text-indigo-600");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$k, 34, 6, 748);
				attr_dev(div0, "class", "mx-auto flex items-center justify-center h-12 w-12 rounded-full bg-indigo-100");
				add_location(div0, file$k, 33, 4, 650);
				attr_dev(h2, "class", "mt-4 text-2xl font-bold text-gray-900");
				add_location(h2, file$k, 39, 4, 1059);
				attr_dev(p0, "class", "mt-2 text-sm text-gray-600");
				add_location(p0, file$k, 42, 4, 1156);
				attr_dev(div1, "class", "text-center");
				add_location(div1, file$k, 32, 2, 620);
				attr_dev(label, "for", "verification-code");
				attr_dev(label, "class", "block text-sm font-medium text-gray-700 text-center mb-2");
				add_location(label, file$k, 55, 6, 1477);
				attr_dev(input, "id", "verification-code");
				attr_dev(input, "type", "text");
				attr_dev(input, "maxlength", "6");
				attr_dev(input, "pattern", "[0-9]" + 6);
				attr_dev(input, "placeholder", "000000");
				attr_dev(input, "autocomplete", "one-time-code");
				attr_dev(input, "class", "block w-full text-center text-2xl font-mono px-3 py-3 border border-gray-300 rounded-md placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500");
				toggle_class(input, "border-red-300", /*verificationCode*/ ctx[3].length > 0 && !/*codeValid*/ ctx[4]);
				toggle_class(input, "border-green-300", /*codeValid*/ ctx[4]);
				add_location(input, file$k, 59, 8, 1662);
				attr_dev(div2, "class", "max-w-xs mx-auto");
				add_location(div2, file$k, 58, 6, 1623);
				add_location(div3, file$k, 54, 4, 1465);
				attr_dev(p1, "class", "text-xs text-gray-500");
				add_location(p1, file$k, 119, 6, 4271);
				attr_dev(button0, "type", "button");
				attr_dev(button0, "class", "text-sm text-indigo-600 hover:text-indigo-500");
				add_location(button0, file$k, 124, 8, 4432);
				attr_dev(button1, "type", "button");
				attr_dev(button1, "class", "text-sm text-gray-500 hover:text-gray-700");
				add_location(button1, file$k, 133, 10, 4678);
				add_location(div4, file$k, 132, 8, 4662);
				attr_dev(div5, "class", "space-y-1");
				add_location(div5, file$k, 123, 6, 4400);
				attr_dev(div6, "class", "text-center space-y-2");
				add_location(div6, file$k, 118, 4, 4229);
				attr_dev(div7, "class", "space-y-4");
				add_location(div7, file$k, 52, 2, 1413);
				attr_dev(h4, "class", "text-sm font-medium text-gray-900 mb-2");
				add_location(h4, file$k, 147, 4, 4990);
				add_location(li0, file$k, 151, 6, 5129);
				add_location(li1, file$k, 152, 6, 5186);
				add_location(li2, file$k, 153, 6, 5255);
				attr_dev(ul, "class", "text-xs text-gray-600 space-y-1");
				add_location(ul, file$k, 150, 4, 5078);
				attr_dev(div8, "class", "bg-gray-50 rounded-lg p-4");
				add_location(div8, file$k, 146, 2, 4946);
				attr_dev(div9, "class", "max-w-md mx-auto space-y-6");
				add_location(div9, file$k, 31, 0, 577);
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				insert_dev(target, div9, anchor);
				append_dev(div9, div1);
				append_dev(div1, div0);
				append_dev(div0, svg);
				append_dev(svg, path);
				append_dev(div1, t0);
				append_dev(div1, h2);
				append_dev(div1, t2);
				append_dev(div1, p0);
				append_dev(div1, t4);
				if (if_block0) if_block0.m(div1, null);
				append_dev(div9, t5);
				append_dev(div9, div7);
				append_dev(div7, div3);
				append_dev(div3, label);
				append_dev(div3, t7);
				append_dev(div3, div2);
				append_dev(div2, input);
				set_input_value(input, /*verificationCode*/ ctx[3]);
				append_dev(div2, t8);
				if (if_block1) if_block1.m(div2, null);
				append_dev(div7, t9);
				if (if_block2) if_block2.m(div7, null);
				append_dev(div7, t10);
				if (if_block3) if_block3.m(div7, null);
				append_dev(div7, t11);
				if (if_block4) if_block4.m(div7, null);
				append_dev(div7, t12);
				append_dev(div7, div6);
				append_dev(div6, p1);
				append_dev(div6, t14);
				append_dev(div6, div5);
				append_dev(div5, button0);
				append_dev(div5, t16);
				append_dev(div5, div4);
				append_dev(div4, button1);
				append_dev(div9, t18);
				append_dev(div9, div8);
				append_dev(div8, h4);
				append_dev(div8, t20);
				append_dev(div8, ul);
				append_dev(ul, li0);
				append_dev(ul, t22);
				append_dev(ul, li1);
				append_dev(ul, t24);
				append_dev(ul, li2);

				if (!mounted) {
					dispose = [
						listen_dev(input, "input", /*input_input_handler*/ ctx[8]),
						listen_dev(button0, "click", /*click_handler*/ ctx[9], false, false, false, false),
						listen_dev(button1, "click", /*handleBack*/ ctx[7], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, [dirty]) {
				if (/*email*/ ctx[2]) {
					if (if_block0) {
						if_block0.p(ctx, dirty);
					} else {
						if_block0 = create_if_block_4$c(ctx);
						if_block0.c();
						if_block0.m(div1, null);
					}
				} else if (if_block0) {
					if_block0.d(1);
					if_block0 = null;
				}

				if (dirty & /*verificationCode*/ 8 && input.value !== /*verificationCode*/ ctx[3]) {
					set_input_value(input, /*verificationCode*/ ctx[3]);
				}

				if (dirty & /*verificationCode, codeValid*/ 24) {
					toggle_class(input, "border-red-300", /*verificationCode*/ ctx[3].length > 0 && !/*codeValid*/ ctx[4]);
				}

				if (dirty & /*codeValid*/ 16) {
					toggle_class(input, "border-green-300", /*codeValid*/ ctx[4]);
				}

				if (/*verificationCode*/ ctx[3].length > 0 && !/*codeValid*/ ctx[4]) {
					if (if_block1) ; else {
						if_block1 = create_if_block_3$d(ctx);
						if_block1.c();
						if_block1.m(div2, null);
					}
				} else if (if_block1) {
					if_block1.d(1);
					if_block1 = null;
				}

				if (/*error*/ ctx[1]) {
					if (if_block2) {
						if_block2.p(ctx, dirty);
					} else {
						if_block2 = create_if_block_2$e(ctx);
						if_block2.c();
						if_block2.m(div7, t10);
					}
				} else if (if_block2) {
					if_block2.d(1);
					if_block2 = null;
				}

				if (/*isLoading*/ ctx[0]) {
					if (if_block3) ; else {
						if_block3 = create_if_block_1$h(ctx);
						if_block3.c();
						if_block3.m(div7, t11);
					}
				} else if (if_block3) {
					if_block3.d(1);
					if_block3 = null;
				}

				if (/*codeValid*/ ctx[4] && !/*isLoading*/ ctx[0]) {
					if (if_block4) {
						if_block4.p(ctx, dirty);
					} else {
						if_block4 = create_if_block$i(ctx);
						if_block4.c();
						if_block4.m(div7, t12);
					}
				} else if (if_block4) {
					if_block4.d(1);
					if_block4 = null;
				}
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
				if (if_block4) if_block4.d();
				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_fragment$k.name,
			type: "component",
			source: "",
			ctx
		});

		return block;
	}

	function instance$k($$self, $$props, $$invalidate) {
		let codeValid;
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('TwoFactorVerification', slots, []);
		const dispatch = createEventDispatcher();
		let { isLoading = false } = $$props;
		let { error = '' } = $$props;
		let { email = '' } = $$props;
		let verificationCode = '';

		function handleSubmit() {
			if (!codeValid) return;
			dispatch('verify', { code: verificationCode });
		}

		function handleBack() {
			dispatch('back');
		}

		const writable_props = ['isLoading', 'error', 'email'];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<TwoFactorVerification> was created with unknown prop '${key}'`);
		});

		function input_input_handler() {
			verificationCode = this.value;
			$$invalidate(3, verificationCode);
		}

		const click_handler = () => dispatch('resend');

		$$self.$$set = $$props => {
			if ('isLoading' in $$props) $$invalidate(0, isLoading = $$props.isLoading);
			if ('error' in $$props) $$invalidate(1, error = $$props.error);
			if ('email' in $$props) $$invalidate(2, email = $$props.email);
		};

		$$self.$capture_state = () => ({
			createEventDispatcher,
			dispatch,
			isLoading,
			error,
			email,
			verificationCode,
			handleSubmit,
			handleBack,
			codeValid
		});

		$$self.$inject_state = $$props => {
			if ('isLoading' in $$props) $$invalidate(0, isLoading = $$props.isLoading);
			if ('error' in $$props) $$invalidate(1, error = $$props.error);
			if ('email' in $$props) $$invalidate(2, email = $$props.email);
			if ('verificationCode' in $$props) $$invalidate(3, verificationCode = $$props.verificationCode);
			if ('codeValid' in $$props) $$invalidate(4, codeValid = $$props.codeValid);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		$$self.$$.update = () => {
			if ($$self.$$.dirty & /*verificationCode*/ 8) {
				$$invalidate(4, codeValid = verificationCode.length === 6 && (/^\d{6}$/).test(verificationCode));
			}

			if ($$self.$$.dirty & /*codeValid, isLoading*/ 17) {
				// Auto-submit when 6 digits are entered
				if (codeValid && !isLoading) {
					handleSubmit();
				}
			}
		};

		return [
			isLoading,
			error,
			email,
			verificationCode,
			codeValid,
			dispatch,
			handleSubmit,
			handleBack,
			input_input_handler,
			click_handler
		];
	}

	class TwoFactorVerification extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance$k, create_fragment$k, safe_not_equal, { isLoading: 0, error: 1, email: 2 });

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "TwoFactorVerification",
				options,
				id: create_fragment$k.name
			});
		}

		get isLoading() {
			throw new Error("<TwoFactorVerification>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}

		set isLoading(value) {
			throw new Error("<TwoFactorVerification>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}

		get error() {
			throw new Error("<TwoFactorVerification>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}

		set error(value) {
			throw new Error("<TwoFactorVerification>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}

		get email() {
			throw new Error("<TwoFactorVerification>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}

		set email(value) {
			throw new Error("<TwoFactorVerification>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}
	}

	/* src/lib/components/Login.svelte generated by Svelte v4.2.20 */
	const file$j = "src/lib/components/Login.svelte";

	// (197:35) 
	function create_if_block_3$c(ctx) {
		let twofactorsetup;
		let current;

		let twofactorsetup_props = {
			qrCodeUrl: /*qrCodeUrl*/ ctx[4],
			secret: /*secret*/ ctx[5],
			isLoading: /*isLoading*/ ctx[1],
			error: /*error*/ ctx[2]
		};

		twofactorsetup = new TwoFactorSetup({
				props: twofactorsetup_props,
				$$inline: true
			});

		/*twofactorsetup_binding*/ ctx[19](twofactorsetup);
		twofactorsetup.$on("verify", /*handle2FASetupVerification*/ ctx[12]);
		twofactorsetup.$on("cancel", /*cancel2FASetup*/ ctx[16]);
		twofactorsetup.$on("complete", /*complete2FASetup*/ ctx[17]);

		const block = {
			c: function create() {
				create_component(twofactorsetup.$$.fragment);
			},
			m: function mount(target, anchor) {
				mount_component(twofactorsetup, target, anchor);
				current = true;
			},
			p: function update(ctx, dirty) {
				const twofactorsetup_changes = {};
				if (dirty & /*qrCodeUrl*/ 16) twofactorsetup_changes.qrCodeUrl = /*qrCodeUrl*/ ctx[4];
				if (dirty & /*secret*/ 32) twofactorsetup_changes.secret = /*secret*/ ctx[5];
				if (dirty & /*isLoading*/ 2) twofactorsetup_changes.isLoading = /*isLoading*/ ctx[1];
				if (dirty & /*error*/ 4) twofactorsetup_changes.error = /*error*/ ctx[2];
				twofactorsetup.$set(twofactorsetup_changes);
			},
			i: function intro(local) {
				if (current) return;
				transition_in(twofactorsetup.$$.fragment, local);
				current = true;
			},
			o: function outro(local) {
				transition_out(twofactorsetup.$$.fragment, local);
				current = false;
			},
			d: function destroy(detaching) {
				/*twofactorsetup_binding*/ ctx[19](null);
				destroy_component(twofactorsetup, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_3$c.name,
			type: "if",
			source: "(197:35) ",
			ctx
		});

		return block;
	}

	// (183:36) 
	function create_if_block_2$d(ctx) {
		let twofactorverification;
		let current;

		twofactorverification = new TwoFactorVerification({
				props: {
					isLoading: /*isLoading*/ ctx[1],
					error: /*error*/ ctx[2],
					email: /*pendingEmail*/ ctx[6]
				},
				$$inline: true
			});

		twofactorverification.$on("verify", /*handle2FAVerification*/ ctx[10]);
		twofactorverification.$on("back", /*backToLogin*/ ctx[15]);
		twofactorverification.$on("resend", /*resend_handler*/ ctx[18]);

		const block = {
			c: function create() {
				create_component(twofactorverification.$$.fragment);
			},
			m: function mount(target, anchor) {
				mount_component(twofactorverification, target, anchor);
				current = true;
			},
			p: function update(ctx, dirty) {
				const twofactorverification_changes = {};
				if (dirty & /*isLoading*/ 2) twofactorverification_changes.isLoading = /*isLoading*/ ctx[1];
				if (dirty & /*error*/ 4) twofactorverification_changes.error = /*error*/ ctx[2];
				if (dirty & /*pendingEmail*/ 64) twofactorverification_changes.email = /*pendingEmail*/ ctx[6];
				twofactorverification.$set(twofactorverification_changes);
			},
			i: function intro(local) {
				if (current) return;
				transition_in(twofactorverification.$$.fragment, local);
				current = true;
			},
			o: function outro(local) {
				transition_out(twofactorverification.$$.fragment, local);
				current = false;
			},
			d: function destroy(detaching) {
				destroy_component(twofactorverification, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_2$d.name,
			type: "if",
			source: "(183:36) ",
			ctx
		});

		return block;
	}

	// (174:34) 
	function create_if_block_1$g(ctx) {
		let registerform;
		let current;

		registerform = new RegisterForm({
				props: {
					isLoading: /*isLoading*/ ctx[1],
					error: /*error*/ ctx[2],
					success: /*success*/ ctx[3]
				},
				$$inline: true
			});

		registerform.$on("register", /*handleRegister*/ ctx[9]);
		registerform.$on("switchMode", /*switchToLogin*/ ctx[13]);

		const block = {
			c: function create() {
				create_component(registerform.$$.fragment);
			},
			m: function mount(target, anchor) {
				mount_component(registerform, target, anchor);
				current = true;
			},
			p: function update(ctx, dirty) {
				const registerform_changes = {};
				if (dirty & /*isLoading*/ 2) registerform_changes.isLoading = /*isLoading*/ ctx[1];
				if (dirty & /*error*/ 4) registerform_changes.error = /*error*/ ctx[2];
				if (dirty & /*success*/ 8) registerform_changes.success = /*success*/ ctx[3];
				registerform.$set(registerform_changes);
			},
			i: function intro(local) {
				if (current) return;
				transition_in(registerform.$$.fragment, local);
				current = true;
			},
			o: function outro(local) {
				transition_out(registerform.$$.fragment, local);
				current = false;
			},
			d: function destroy(detaching) {
				destroy_component(registerform, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_1$g.name,
			type: "if",
			source: "(174:34) ",
			ctx
		});

		return block;
	}

	// (155:4) {#if mode === 'login'}
	function create_if_block$h(ctx) {
		let loginform;
		let t0;
		let div;
		let button;
		let current;
		let mounted;
		let dispose;

		loginform = new LoginForm({
				props: {
					isLoading: /*isLoading*/ ctx[1],
					error: /*error*/ ctx[2]
				},
				$$inline: true
			});

		loginform.$on("login", /*handleLogin*/ ctx[8]);
		loginform.$on("switchMode", /*switchToRegister*/ ctx[14]);

		const block = {
			c: function create() {
				create_component(loginform.$$.fragment);
				t0 = space();
				div = element("div");
				button = element("button");
				button.textContent = "Set up Two-Factor Authentication";
				attr_dev(button, "type", "button");
				attr_dev(button, "class", "text-sm text-gray-500 hover:text-gray-700");
				add_location(button, file$j, 174, 8, 4488);
				attr_dev(div, "class", "mt-6 text-center");
				add_location(div, file$j, 173, 6, 4449);
			},
			m: function mount(target, anchor) {
				mount_component(loginform, target, anchor);
				insert_dev(target, t0, anchor);
				insert_dev(target, div, anchor);
				append_dev(div, button);
				current = true;

				if (!mounted) {
					dispose = listen_dev(button, "click", /*handle2FASetup*/ ctx[11], false, false, false, false);
					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				const loginform_changes = {};
				if (dirty & /*isLoading*/ 2) loginform_changes.isLoading = /*isLoading*/ ctx[1];
				if (dirty & /*error*/ 4) loginform_changes.error = /*error*/ ctx[2];
				loginform.$set(loginform_changes);
			},
			i: function intro(local) {
				if (current) return;
				transition_in(loginform.$$.fragment, local);
				current = true;
			},
			o: function outro(local) {
				transition_out(loginform.$$.fragment, local);
				current = false;
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(t0);
					detach_dev(div);
				}

				destroy_component(loginform, detaching);
				mounted = false;
				dispose();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block$h.name,
			type: "if",
			source: "(155:4) {#if mode === 'login'}",
			ctx
		});

		return block;
	}

	function create_fragment$j(ctx) {
		let div1;
		let div0;
		let current_block_type_index;
		let if_block;
		let current;
		const if_block_creators = [create_if_block$h, create_if_block_1$g, create_if_block_2$d, create_if_block_3$c];
		const if_blocks = [];

		function select_block_type(ctx, dirty) {
			if (/*mode*/ ctx[0] === 'login') return 0;
			if (/*mode*/ ctx[0] === 'register') return 1;
			if (/*mode*/ ctx[0] === '2fa-verify') return 2;
			if (/*mode*/ ctx[0] === '2fa-setup') return 3;
			return -1;
		}

		if (~(current_block_type_index = select_block_type(ctx))) {
			if_block = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);
		}

		const block = {
			c: function create() {
				div1 = element("div");
				div0 = element("div");
				if (if_block) if_block.c();
				attr_dev(div0, "class", "max-w-md w-full");
				add_location(div0, file$j, 163, 2, 4184);
				attr_dev(div1, "class", "min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8");
				add_location(div1, file$j, 162, 0, 4084);
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				insert_dev(target, div1, anchor);
				append_dev(div1, div0);

				if (~current_block_type_index) {
					if_blocks[current_block_type_index].m(div0, null);
				}

				current = true;
			},
			p: function update(ctx, [dirty]) {
				let previous_block_index = current_block_type_index;
				current_block_type_index = select_block_type(ctx);

				if (current_block_type_index === previous_block_index) {
					if (~current_block_type_index) {
						if_blocks[current_block_type_index].p(ctx, dirty);
					}
				} else {
					if (if_block) {
						group_outros();

						transition_out(if_blocks[previous_block_index], 1, 1, () => {
							if_blocks[previous_block_index] = null;
						});

						check_outros();
					}

					if (~current_block_type_index) {
						if_block = if_blocks[current_block_type_index];

						if (!if_block) {
							if_block = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);
							if_block.c();
						} else {
							if_block.p(ctx, dirty);
						}

						transition_in(if_block, 1);
						if_block.m(div0, null);
					} else {
						if_block = null;
					}
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
					detach_dev(div1);
				}

				if (~current_block_type_index) {
					if_blocks[current_block_type_index].d();
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_fragment$j.name,
			type: "component",
			source: "",
			ctx
		});

		return block;
	}

	function instance$j($$self, $$props, $$invalidate) {
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('Login', slots, []);
		let mode = 'login'; // 'login', 'register', '2fa-setup', '2fa-verify'
		let isLoading = false;
		let error = '';
		let success = '';

		// 2FA setup data
		let qrCodeUrl = '';

		let secret = '';
		let pendingEmail = '';

		// Component references
		let twoFactorSetupRef;

		async function handleLogin(event) {
			const { email, password, totpCode } = event.detail;
			$$invalidate(1, isLoading = true);
			$$invalidate(2, error = '');

			try {
				const result = await authActions.login(email, password, totpCode);

				if (!result.success) {
					if (result.message?.toLowerCase().includes('2fa') || result.message?.toLowerCase().includes('totp')) {
						$$invalidate(6, pendingEmail = email);
						$$invalidate(0, mode = '2fa-verify');
					} else {
						$$invalidate(2, error = result.message || 'Login failed');
					}
				}
			} catch(err) {
				$$invalidate(
					2,
					error = 'An unexpected error occurred'
				);
			} finally {
				$$invalidate(1, isLoading = false);
			}
		}

		async function handleRegister(event) {
			const { email, password } = event.detail;
			$$invalidate(1, isLoading = true);
			$$invalidate(2, error = '');
			$$invalidate(3, success = '');

			try {
				const result = await authActions.register(email, password);

				if (result.success) {
					$$invalidate(3, success = 'Account created successfully! You can now sign in.');
					$$invalidate(0, mode = 'login');
				} else {
					$$invalidate(2, error = result.message || 'Registration failed');
				}
			} catch(err) {
				$$invalidate(2, error = 'An unexpected error occurred');
			} finally {
				$$invalidate(1, isLoading = false);
			}
		}

		async function handle2FAVerification(event) {
			const { code } = event.detail;
			$$invalidate(1, isLoading = true);
			$$invalidate(2, error = '');

			try {
				// For login verification, we need to retry the login with the TOTP code
				// This assumes we stored the email/password from the previous attempt
				// In a real implementation, you might want to handle this differently
				const result = await authActions.verify2FA(code);

				if (!result.success) {
					$$invalidate(2, error = result.message || 'Invalid verification code');
				}
			} catch(err) {
				$$invalidate(
					2,
					error = 'An unexpected error occurred'
				);
			} finally {
				$$invalidate(1, isLoading = false);
			}
		}

		async function handle2FASetup() {
			$$invalidate(1, isLoading = true);
			$$invalidate(2, error = '');

			try {
				const result = await authActions.setup2FA();

				if (result.success) {
					$$invalidate(4, qrCodeUrl = result.qrCodeUrl);
					$$invalidate(5, secret = result.secret);
					$$invalidate(0, mode = '2fa-setup');
				} else {
					$$invalidate(2, error = result.message || 'Failed to setup 2FA');
				}
			} catch(err) {
				$$invalidate(2, error = 'An unexpected error occurred');
			} finally {
				$$invalidate(1, isLoading = false);
			}
		}

		async function handle2FASetupVerification(event) {
			const { code } = event.detail;
			$$invalidate(1, isLoading = true);
			$$invalidate(2, error = '');

			try {
				const result = await authActions.verify2FA(code);

				if (result.success) {
					twoFactorSetupRef?.showSuccess();
				} else {
					$$invalidate(2, error = result.message || 'Invalid verification code');
				}
			} catch(err) {
				$$invalidate(2, error = 'An unexpected error occurred');
			} finally {
				$$invalidate(1, isLoading = false);
			}
		}

		function switchToLogin() {
			$$invalidate(0, mode = 'login');
			$$invalidate(2, error = '');
			$$invalidate(3, success = '');
		}

		function switchToRegister() {
			$$invalidate(0, mode = 'register');
			$$invalidate(2, error = '');
			$$invalidate(3, success = '');
		}

		function backToLogin() {
			$$invalidate(0, mode = 'login');
			$$invalidate(2, error = '');
			$$invalidate(6, pendingEmail = '');
		}

		function cancel2FASetup() {
			$$invalidate(0, mode = 'login');
			$$invalidate(2, error = '');
			$$invalidate(4, qrCodeUrl = '');
			$$invalidate(5, secret = '');
		}

		function complete2FASetup() {
			// 2FA setup complete, user should now be authenticated
			$$invalidate(0, mode = 'login');

			$$invalidate(4, qrCodeUrl = '');
			$$invalidate(5, secret = '');
		}

		const writable_props = [];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<Login> was created with unknown prop '${key}'`);
		});

		const resend_handler = () => {
			// Could implement resend logic here
			$$invalidate(2, error = 'Please try logging in again');

			backToLogin();
		};

		function twofactorsetup_binding($$value) {
			binding_callbacks[$$value ? 'unshift' : 'push'](() => {
				twoFactorSetupRef = $$value;
				$$invalidate(7, twoFactorSetupRef);
			});
		}

		$$self.$capture_state = () => ({
			authActions,
			LoginForm,
			RegisterForm,
			TwoFactorSetup,
			TwoFactorVerification,
			mode,
			isLoading,
			error,
			success,
			qrCodeUrl,
			secret,
			pendingEmail,
			twoFactorSetupRef,
			handleLogin,
			handleRegister,
			handle2FAVerification,
			handle2FASetup,
			handle2FASetupVerification,
			switchToLogin,
			switchToRegister,
			backToLogin,
			cancel2FASetup,
			complete2FASetup
		});

		$$self.$inject_state = $$props => {
			if ('mode' in $$props) $$invalidate(0, mode = $$props.mode);
			if ('isLoading' in $$props) $$invalidate(1, isLoading = $$props.isLoading);
			if ('error' in $$props) $$invalidate(2, error = $$props.error);
			if ('success' in $$props) $$invalidate(3, success = $$props.success);
			if ('qrCodeUrl' in $$props) $$invalidate(4, qrCodeUrl = $$props.qrCodeUrl);
			if ('secret' in $$props) $$invalidate(5, secret = $$props.secret);
			if ('pendingEmail' in $$props) $$invalidate(6, pendingEmail = $$props.pendingEmail);
			if ('twoFactorSetupRef' in $$props) $$invalidate(7, twoFactorSetupRef = $$props.twoFactorSetupRef);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		return [
			mode,
			isLoading,
			error,
			success,
			qrCodeUrl,
			secret,
			pendingEmail,
			twoFactorSetupRef,
			handleLogin,
			handleRegister,
			handle2FAVerification,
			handle2FASetup,
			handle2FASetupVerification,
			switchToLogin,
			switchToRegister,
			backToLogin,
			cancel2FASetup,
			complete2FASetup,
			resend_handler,
			twofactorsetup_binding
		];
	}

	class Login extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance$j, create_fragment$j, safe_not_equal, {});

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "Login",
				options,
				id: create_fragment$j.name
			});
		}
	}

	const initialState$3 = {
	    connections: [],
	    isLoading: false,
	    error: null,
	};
	const connectionsStore = writable(initialState$3);
	const connectedServices = derived(connectionsStore, ($connections) => $connections.connections.filter(conn => conn.status === 'active'));
	const spotifyConnection = derived(connectionsStore, ($connections) => $connections.connections.find(conn => conn.provider === 'spotify'));
	const hasActiveSpotifyConnection = derived(spotifyConnection, ($spotify) => $spotify?.status === 'active');
	// Connection actions
	const connectionActions = {
	    fetchConnections: async () => {
	        connectionsStore.update(state => ({ ...state, isLoading: true, error: null }));
	        try {
	            const token = localStorage.getItem('auth_token');
	            const response = await fetch('http://localhost:3000/api/v1/connections', {
	                headers: {
	                    'Authorization': `Bearer ${token}`,
	                },
	            });
	            const result = await response.json();
	            if (result.success) {
	                connectionsStore.update(state => ({
	                    ...state,
	                    connections: result.data,
	                    isLoading: false,
	                }));
	            }
	            else {
	                connectionsStore.update(state => ({
	                    ...state,
	                    error: result.message,
	                    isLoading: false,
	                }));
	            }
	        }
	        catch (error) {
	            connectionsStore.update(state => ({
	                ...state,
	                error: 'Failed to fetch connections',
	                isLoading: false,
	            }));
	        }
	    },
	    initiateSpotifyAuth: async () => {
	        try {
	            const token = localStorage.getItem('auth_token');
	            const response = await fetch('http://localhost:3000/api/v1/spotify/auth', {
	                headers: {
	                    'Authorization': `Bearer ${token}`,
	                },
	            });
	            const result = await response.json();
	            if (result.success) {
	                // Redirect to Spotify authorization
	                window.location.href = result.data.auth_url;
	            }
	            else {
	                throw new Error(result.message);
	            }
	        }
	        catch (error) {
	            connectionsStore.update(state => ({
	                ...state,
	                error: `Failed to initiate Spotify auth: ${error instanceof Error ? error.message : 'Unknown error'}`,
	            }));
	        }
	    },
	    handleSpotifyCallback: async (code, state) => {
	        try {
	            const token = localStorage.getItem('auth_token');
	            const response = await fetch('http://localhost:3000/api/v1/spotify/callback', {
	                method: 'POST',
	                headers: {
	                    'Authorization': `Bearer ${token}`,
	                    'Content-Type': 'application/json',
	                },
	                body: JSON.stringify({ code, state }),
	            });
	            const result = await response.json();
	            if (result.success) {
	                // Refresh connections
	                await connectionActions.fetchConnections();
	                return { success: true };
	            }
	            else {
	                return { success: false, message: result.message };
	            }
	        }
	        catch (error) {
	            return { success: false, message: 'Failed to complete Spotify connection' };
	        }
	    },
	    disconnectSpotify: async () => {
	        try {
	            const token = localStorage.getItem('auth_token');
	            const response = await fetch('http://localhost:3000/api/v1/spotify/connection', {
	                method: 'DELETE',
	                headers: {
	                    'Authorization': `Bearer ${token}`,
	                },
	            });
	            const result = await response.json();
	            if (result.success) {
	                // Refresh connections
	                await connectionActions.fetchConnections();
	                return { success: true };
	            }
	            else {
	                return { success: false, message: result.message };
	            }
	        }
	        catch (error) {
	            return { success: false, message: 'Failed to disconnect Spotify' };
	        }
	    },
	    checkSpotifyHealth: async () => {
	        try {
	            const token = localStorage.getItem('auth_token');
	            const response = await fetch('http://localhost:3000/api/v1/spotify/health', {
	                headers: {
	                    'Authorization': `Bearer ${token}`,
	                },
	            });
	            const result = await response.json();
	            if (result.success) {
	                // Update connection status if needed
	                await connectionActions.fetchConnections();
	                return result.data;
	            }
	        }
	        catch (error) {
	            console.error('Spotify health check failed:', error);
	        }
	    },
	};

	const initialState$2 = {
	    entries: [],
	    isLoading: false,
	    error: null,
	    searchResults: [],
	    isSearching: false,
	};
	const dnpStore = writable(initialState$2);
	derived(dnpStore, ($dnp) => $dnp.entries.map(entry => entry.artist));
	const dnpCount = derived(dnpStore, ($dnp) => $dnp.entries.length);
	const dnpTags = derived(dnpStore, ($dnp) => {
	    const allTags = $dnp.entries.flatMap(entry => entry.tags);
	    return [...new Set(allTags)].sort();
	});
	// DNP actions
	const dnpActions = {
	    fetchDnpList: async () => {
	        dnpStore.update(state => ({ ...state, isLoading: true, error: null }));
	        try {
	            const token = localStorage.getItem('auth_token');
	            const response = await fetch('http://localhost:3000/api/v1/dnp/list', {
	                headers: {
	                    'Authorization': `Bearer ${token}`,
	                },
	            });
	            const result = await response.json();
	            if (result.success) {
	                dnpStore.update(state => ({
	                    ...state,
	                    entries: result.data,
	                    isLoading: false,
	                }));
	            }
	            else {
	                dnpStore.update(state => ({
	                    ...state,
	                    error: result.message,
	                    isLoading: false,
	                }));
	            }
	        }
	        catch (error) {
	            dnpStore.update(state => ({
	                ...state,
	                error: 'Failed to fetch DNP list',
	                isLoading: false,
	            }));
	        }
	    },
	    searchArtists: async (query, limit = 10) => {
	        if (!query.trim()) {
	            dnpStore.update(state => ({ ...state, searchResults: [] }));
	            return;
	        }
	        dnpStore.update(state => ({ ...state, isSearching: true }));
	        try {
	            const token = localStorage.getItem('auth_token');
	            const response = await fetch(`http://localhost:3000/api/v1/dnp/search?q=${encodeURIComponent(query)}&limit=${limit}`, {
	                headers: {
	                    'Authorization': `Bearer ${token}`,
	                },
	            });
	            const result = await response.json();
	            if (result.success) {
	                dnpStore.update(state => ({
	                    ...state,
	                    searchResults: result.data,
	                    isSearching: false,
	                }));
	            }
	            else {
	                dnpStore.update(state => ({
	                    ...state,
	                    error: result.message,
	                    isSearching: false,
	                }));
	            }
	        }
	        catch (error) {
	            dnpStore.update(state => ({
	                ...state,
	                error: 'Artist search failed',
	                isSearching: false,
	            }));
	        }
	    },
	    addArtist: async (artistQuery, tags = [], note) => {
	        try {
	            const token = localStorage.getItem('auth_token');
	            const response = await fetch('http://localhost:3000/api/v1/dnp/artists', {
	                method: 'POST',
	                headers: {
	                    'Authorization': `Bearer ${token}`,
	                    'Content-Type': 'application/json',
	                },
	                body: JSON.stringify({
	                    query: artistQuery,
	                    tags,
	                    note,
	                }),
	            });
	            const result = await response.json();
	            if (result.success) {
	                // Refresh the DNP list
	                await dnpActions.fetchDnpList();
	                return { success: true, data: result.data };
	            }
	            else {
	                return { success: false, message: result.message };
	            }
	        }
	        catch (error) {
	            return { success: false, message: 'Failed to add artist to DNP list' };
	        }
	    },
	    removeArtist: async (artistId) => {
	        try {
	            const token = localStorage.getItem('auth_token');
	            const response = await fetch(`http://localhost:3000/api/v1/dnp/artists/${artistId}`, {
	                method: 'DELETE',
	                headers: {
	                    'Authorization': `Bearer ${token}`,
	                },
	            });
	            const result = await response.json();
	            if (result.success) {
	                // Refresh the DNP list
	                await dnpActions.fetchDnpList();
	                return { success: true };
	            }
	            else {
	                return { success: false, message: result.message };
	            }
	        }
	        catch (error) {
	            return { success: false, message: 'Failed to remove artist from DNP list' };
	        }
	    },
	    updateEntry: async (artistId, tags, note) => {
	        try {
	            const token = localStorage.getItem('auth_token');
	            const response = await fetch(`http://localhost:3000/api/v1/dnp/artists/${artistId}`, {
	                method: 'PUT',
	                headers: {
	                    'Authorization': `Bearer ${token}`,
	                    'Content-Type': 'application/json',
	                },
	                body: JSON.stringify({ tags, note }),
	            });
	            const result = await response.json();
	            if (result.success) {
	                // Refresh the DNP list
	                await dnpActions.fetchDnpList();
	                return { success: true, data: result.data };
	            }
	            else {
	                return { success: false, message: result.message };
	            }
	        }
	        catch (error) {
	            return { success: false, message: 'Failed to update DNP entry' };
	        }
	    },
	    bulkImport: async (data, format) => {
	        try {
	            const token = localStorage.getItem('auth_token');
	            const response = await fetch('http://localhost:3000/api/v1/dnp/import', {
	                method: 'POST',
	                headers: {
	                    'Authorization': `Bearer ${token}`,
	                    'Content-Type': 'application/json',
	                },
	                body: JSON.stringify({ data, format }),
	            });
	            const result = await response.json();
	            if (result.success) {
	                // Refresh the DNP list
	                await dnpActions.fetchDnpList();
	                return { success: true, data: result.data };
	            }
	            else {
	                return { success: false, message: result.message };
	            }
	        }
	        catch (error) {
	            return { success: false, message: 'Bulk import failed' };
	        }
	    },
	    exportList: async (format = 'json') => {
	        try {
	            const token = localStorage.getItem('auth_token');
	            const response = await fetch(`http://localhost:3000/api/v1/dnp/export?format=${format}`, {
	                headers: {
	                    'Authorization': `Bearer ${token}`,
	                },
	            });
	            const result = await response.json();
	            if (result.success) {
	                return { success: true, data: result.data };
	            }
	            else {
	                return { success: false, message: result.message };
	            }
	        }
	        catch (error) {
	            return { success: false, message: 'Export failed' };
	        }
	    },
	    clearSearch: () => {
	        dnpStore.update(state => ({ ...state, searchResults: [] }));
	    },
	};

	/* src/lib/components/Navigation.svelte generated by Svelte v4.2.20 */
	const file$i = "src/lib/components/Navigation.svelte";

	function create_fragment$i(ctx) {
		let nav0;
		let div3;
		let div2;
		let div0;
		let h1;
		let t1;
		let div1;
		let span;
		let t2_value = /*$currentUser*/ ctx[0]?.email + "";
		let t2;
		let t3;
		let button0;
		let t5;
		let button1;
		let t7;
		let div5;
		let div4;
		let nav1;
		let button2;
		let t8;
		let button2_class_value;
		let t9;
		let button3;
		let t10;
		let button3_class_value;
		let t11;
		let button4;
		let t12;
		let t13;
		let t14;
		let button4_class_value;
		let t15;
		let button5;
		let t16;
		let button5_class_value;
		let t17;
		let button6;
		let t18;
		let button6_class_value;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				nav0 = element("nav");
				div3 = element("div");
				div2 = element("div");
				div0 = element("div");
				h1 = element("h1");
				h1.textContent = "No Drake in the House";
				t1 = space();
				div1 = element("div");
				span = element("span");
				t2 = text(t2_value);
				t3 = space();
				button0 = element("button");
				button0.textContent = "Settings";
				t5 = space();
				button1 = element("button");
				button1.textContent = "Sign out";
				t7 = space();
				div5 = element("div");
				div4 = element("div");
				nav1 = element("nav");
				button2 = element("button");
				t8 = text("Overview");
				t9 = space();
				button3 = element("button");
				t10 = text("Connections");
				t11 = space();
				button4 = element("button");
				t12 = text("DNP List (");
				t13 = text(/*$dnpCount*/ ctx[2]);
				t14 = text(")");
				t15 = space();
				button5 = element("button");
				t16 = text("Enforcement");
				t17 = space();
				button6 = element("button");
				t18 = text("Community Lists");
				attr_dev(h1, "class", "text-xl font-semibold text-gray-900");
				add_location(h1, file$i, 19, 8, 524);
				attr_dev(div0, "class", "flex items-center");
				add_location(div0, file$i, 18, 6, 484);
				attr_dev(span, "class", "text-sm text-gray-700");
				add_location(span, file$i, 25, 8, 695);
				attr_dev(button0, "class", "text-sm text-gray-500 hover:text-gray-700");
				add_location(button0, file$i, 28, 8, 788);
				attr_dev(button1, "class", "text-sm text-gray-500 hover:text-gray-700");
				add_location(button1, file$i, 34, 8, 958);
				attr_dev(div1, "class", "flex items-center space-x-4");
				add_location(div1, file$i, 24, 6, 645);
				attr_dev(div2, "class", "flex justify-between h-16");
				add_location(div2, file$i, 17, 4, 438);
				attr_dev(div3, "class", "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8");
				add_location(div3, file$i, 16, 2, 381);
				attr_dev(nav0, "class", "bg-white shadow-sm border-b border-gray-200");
				add_location(nav0, file$i, 15, 0, 321);

				attr_dev(button2, "class", button2_class_value = "py-4 px-1 border-b-2 font-medium text-sm " + (/*$currentRoute*/ ctx[1] === 'overview'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'));

				add_location(button2, file$i, 49, 6, 1317);

				attr_dev(button3, "class", button3_class_value = "py-4 px-1 border-b-2 font-medium text-sm " + (/*$currentRoute*/ ctx[1] === 'connections'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'));

				add_location(button3, file$i, 55, 6, 1623);

				attr_dev(button4, "class", button4_class_value = "py-4 px-1 border-b-2 font-medium text-sm " + (/*$currentRoute*/ ctx[1] === 'dnp'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'));

				add_location(button4, file$i, 61, 6, 1938);

				attr_dev(button5, "class", button5_class_value = "py-4 px-1 border-b-2 font-medium text-sm " + (/*$currentRoute*/ ctx[1] === 'enforcement'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'));

				add_location(button5, file$i, 67, 6, 2248);

				attr_dev(button6, "class", button6_class_value = "py-4 px-1 border-b-2 font-medium text-sm " + (/*$currentRoute*/ ctx[1] === 'community'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'));

				add_location(button6, file$i, 73, 6, 2563);
				attr_dev(nav1, "class", "flex space-x-8");
				attr_dev(nav1, "aria-label", "Tabs");
				add_location(nav1, file$i, 48, 4, 1264);
				attr_dev(div4, "class", "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8");
				add_location(div4, file$i, 47, 2, 1207);
				attr_dev(div5, "class", "bg-white shadow-sm");
				add_location(div5, file$i, 46, 0, 1172);
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				insert_dev(target, nav0, anchor);
				append_dev(nav0, div3);
				append_dev(div3, div2);
				append_dev(div2, div0);
				append_dev(div0, h1);
				append_dev(div2, t1);
				append_dev(div2, div1);
				append_dev(div1, span);
				append_dev(span, t2);
				append_dev(div1, t3);
				append_dev(div1, button0);
				append_dev(div1, t5);
				append_dev(div1, button1);
				insert_dev(target, t7, anchor);
				insert_dev(target, div5, anchor);
				append_dev(div5, div4);
				append_dev(div4, nav1);
				append_dev(nav1, button2);
				append_dev(button2, t8);
				append_dev(nav1, t9);
				append_dev(nav1, button3);
				append_dev(button3, t10);
				append_dev(nav1, t11);
				append_dev(nav1, button4);
				append_dev(button4, t12);
				append_dev(button4, t13);
				append_dev(button4, t14);
				append_dev(nav1, t15);
				append_dev(nav1, button5);
				append_dev(button5, t16);
				append_dev(nav1, t17);
				append_dev(nav1, button6);
				append_dev(button6, t18);

				if (!mounted) {
					dispose = [
						listen_dev(button0, "click", /*click_handler*/ ctx[5], false, false, false, false),
						listen_dev(button1, "click", /*handleLogout*/ ctx[3], false, false, false, false),
						listen_dev(button2, "click", /*click_handler_1*/ ctx[6], false, false, false, false),
						listen_dev(button3, "click", /*click_handler_2*/ ctx[7], false, false, false, false),
						listen_dev(button4, "click", /*click_handler_3*/ ctx[8], false, false, false, false),
						listen_dev(button5, "click", /*click_handler_4*/ ctx[9], false, false, false, false),
						listen_dev(button6, "click", /*click_handler_5*/ ctx[10], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, [dirty]) {
				if (dirty & /*$currentUser*/ 1 && t2_value !== (t2_value = /*$currentUser*/ ctx[0]?.email + "")) set_data_dev(t2, t2_value);

				if (dirty & /*$currentRoute*/ 2 && button2_class_value !== (button2_class_value = "py-4 px-1 border-b-2 font-medium text-sm " + (/*$currentRoute*/ ctx[1] === 'overview'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'))) {
					attr_dev(button2, "class", button2_class_value);
				}

				if (dirty & /*$currentRoute*/ 2 && button3_class_value !== (button3_class_value = "py-4 px-1 border-b-2 font-medium text-sm " + (/*$currentRoute*/ ctx[1] === 'connections'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'))) {
					attr_dev(button3, "class", button3_class_value);
				}

				if (dirty & /*$dnpCount*/ 4) set_data_dev(t13, /*$dnpCount*/ ctx[2]);

				if (dirty & /*$currentRoute*/ 2 && button4_class_value !== (button4_class_value = "py-4 px-1 border-b-2 font-medium text-sm " + (/*$currentRoute*/ ctx[1] === 'dnp'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'))) {
					attr_dev(button4, "class", button4_class_value);
				}

				if (dirty & /*$currentRoute*/ 2 && button5_class_value !== (button5_class_value = "py-4 px-1 border-b-2 font-medium text-sm " + (/*$currentRoute*/ ctx[1] === 'enforcement'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'))) {
					attr_dev(button5, "class", button5_class_value);
				}

				if (dirty & /*$currentRoute*/ 2 && button6_class_value !== (button6_class_value = "py-4 px-1 border-b-2 font-medium text-sm " + (/*$currentRoute*/ ctx[1] === 'community'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'))) {
					attr_dev(button6, "class", button6_class_value);
				}
			},
			i: noop,
			o: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(nav0);
					detach_dev(t7);
					detach_dev(div5);
				}

				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_fragment$i.name,
			type: "component",
			source: "",
			ctx
		});

		return block;
	}

	function instance$i($$self, $$props, $$invalidate) {
		let $currentUser;
		let $currentRoute;
		let $dnpCount;
		validate_store(currentUser, 'currentUser');
		component_subscribe($$self, currentUser, $$value => $$invalidate(0, $currentUser = $$value));
		validate_store(currentRoute, 'currentRoute');
		component_subscribe($$self, currentRoute, $$value => $$invalidate(1, $currentRoute = $$value));
		validate_store(dnpCount, 'dnpCount');
		component_subscribe($$self, dnpCount, $$value => $$invalidate(2, $dnpCount = $$value));
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('Navigation', slots, []);

		function handleLogout() {
			authActions.logout();
		}

		function navigate(route) {
			router.navigate(route);
		}

		const writable_props = [];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<Navigation> was created with unknown prop '${key}'`);
		});

		const click_handler = () => navigate('profile');
		const click_handler_1 = () => navigate('overview');
		const click_handler_2 = () => navigate('connections');
		const click_handler_3 = () => navigate('dnp');
		const click_handler_4 = () => navigate('enforcement');
		const click_handler_5 = () => navigate('community');

		$$self.$capture_state = () => ({
			currentRoute,
			router,
			authActions,
			currentUser,
			dnpCount,
			handleLogout,
			navigate,
			$currentUser,
			$currentRoute,
			$dnpCount
		});

		return [
			$currentUser,
			$currentRoute,
			$dnpCount,
			handleLogout,
			navigate,
			click_handler,
			click_handler_1,
			click_handler_2,
			click_handler_3,
			click_handler_4,
			click_handler_5
		];
	}

	class Navigation extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance$i, create_fragment$i, safe_not_equal, {});

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "Navigation",
				options,
				id: create_fragment$i.name
			});
		}
	}

	/* src/lib/components/ServiceConnections.svelte generated by Svelte v4.2.20 */
	const file$h = "src/lib/components/ServiceConnections.svelte";

	// (66:2) {#if error}
	function create_if_block_6$b(ctx) {
		let div3;
		let div2;
		let div0;
		let svg;
		let path;
		let t0;
		let div1;
		let p;
		let t1;

		const block = {
			c: function create() {
				div3 = element("div");
				div2 = element("div");
				div0 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				div1 = element("div");
				p = element("p");
				t1 = text(/*error*/ ctx[1]);
				attr_dev(path, "fill-rule", "evenodd");
				attr_dev(path, "d", "M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z");
				attr_dev(path, "clip-rule", "evenodd");
				add_location(path, file$h, 84, 12, 2322);
				attr_dev(svg, "class", "h-5 w-5 text-red-400");
				attr_dev(svg, "viewBox", "0 0 20 20");
				attr_dev(svg, "fill", "currentColor");
				add_location(svg, file$h, 83, 10, 2235);
				attr_dev(div0, "class", "flex-shrink-0");
				add_location(div0, file$h, 82, 8, 2197);
				attr_dev(p, "class", "text-sm text-red-800");
				add_location(p, file$h, 88, 10, 2660);
				attr_dev(div1, "class", "ml-3");
				add_location(div1, file$h, 87, 8, 2631);
				attr_dev(div2, "class", "flex");
				add_location(div2, file$h, 81, 6, 2170);
				attr_dev(div3, "class", "mb-6 bg-red-50 border border-red-200 rounded-md p-4");
				add_location(div3, file$h, 80, 4, 2098);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div3, anchor);
				append_dev(div3, div2);
				append_dev(div2, div0);
				append_dev(div0, svg);
				append_dev(svg, path);
				append_dev(div2, t0);
				append_dev(div2, div1);
				append_dev(div1, p);
				append_dev(p, t1);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*error*/ 2) set_data_dev(t1, /*error*/ ctx[1]);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div3);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_6$b.name,
			type: "if",
			source: "(66:2) {#if error}",
			ctx
		});

		return block;
	}

	// (97:16) {#if $spotifyConnection}
	function create_if_block_5$b(ctx) {
		let span;
		let t_value = /*$spotifyConnection*/ ctx[2].status + "";
		let t;
		let span_class_value;

		const block = {
			c: function create() {
				span = element("span");
				t = text(t_value);
				attr_dev(span, "class", span_class_value = "ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium " + getStatusColor$2(/*$spotifyConnection*/ ctx[2].status));
				add_location(span, file$h, 111, 18, 4133);
			},
			m: function mount(target, anchor) {
				insert_dev(target, span, anchor);
				append_dev(span, t);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*$spotifyConnection*/ 4 && t_value !== (t_value = /*$spotifyConnection*/ ctx[2].status + "")) set_data_dev(t, t_value);

				if (dirty & /*$spotifyConnection*/ 4 && span_class_value !== (span_class_value = "ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium " + getStatusColor$2(/*$spotifyConnection*/ ctx[2].status))) {
					attr_dev(span, "class", span_class_value);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(span);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_5$b.name,
			type: "if",
			source: "(97:16) {#if $spotifyConnection}",
			ctx
		});

		return block;
	}

	// (116:16) {:else}
	function create_else_block_2$4(ctx) {
		let p;

		const block = {
			c: function create() {
				p = element("p");
				p.textContent = "Connect your Spotify account to manage your music library";
				attr_dev(p, "class", "text-sm text-gray-500");
				add_location(p, file$h, 130, 18, 5053);
			},
			m: function mount(target, anchor) {
				insert_dev(target, p, anchor);
			},
			p: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(p);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block_2$4.name,
			type: "else",
			source: "(116:16) {:else}",
			ctx
		});

		return block;
	}

	// (104:16) {#if $spotifyConnection}
	function create_if_block_2$c(ctx) {
		let p;
		let t0;
		let t1_value = formatDate$6(/*$spotifyConnection*/ ctx[2].created_at) + "";
		let t1;
		let t2;
		let t3;
		let if_block1_anchor;
		let if_block0 = /*$spotifyConnection*/ ctx[2].provider_user_id && create_if_block_4$b(ctx);
		let if_block1 = /*$spotifyConnection*/ ctx[2].scopes.length > 0 && create_if_block_3$b(ctx);

		const block = {
			c: function create() {
				p = element("p");
				t0 = text("Connected ");
				t1 = text(t1_value);
				t2 = space();
				if (if_block0) if_block0.c();
				t3 = space();
				if (if_block1) if_block1.c();
				if_block1_anchor = empty();
				attr_dev(p, "class", "text-sm text-gray-500");
				add_location(p, file$h, 118, 18, 4478);
			},
			m: function mount(target, anchor) {
				insert_dev(target, p, anchor);
				append_dev(p, t0);
				append_dev(p, t1);
				append_dev(p, t2);
				if (if_block0) if_block0.m(p, null);
				insert_dev(target, t3, anchor);
				if (if_block1) if_block1.m(target, anchor);
				insert_dev(target, if_block1_anchor, anchor);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*$spotifyConnection*/ 4 && t1_value !== (t1_value = formatDate$6(/*$spotifyConnection*/ ctx[2].created_at) + "")) set_data_dev(t1, t1_value);

				if (/*$spotifyConnection*/ ctx[2].provider_user_id) {
					if (if_block0) {
						if_block0.p(ctx, dirty);
					} else {
						if_block0 = create_if_block_4$b(ctx);
						if_block0.c();
						if_block0.m(p, null);
					}
				} else if (if_block0) {
					if_block0.d(1);
					if_block0 = null;
				}

				if (/*$spotifyConnection*/ ctx[2].scopes.length > 0) {
					if (if_block1) {
						if_block1.p(ctx, dirty);
					} else {
						if_block1 = create_if_block_3$b(ctx);
						if_block1.c();
						if_block1.m(if_block1_anchor.parentNode, if_block1_anchor);
					}
				} else if (if_block1) {
					if_block1.d(1);
					if_block1 = null;
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(p);
					detach_dev(t3);
					detach_dev(if_block1_anchor);
				}

				if (if_block0) if_block0.d();
				if (if_block1) if_block1.d(detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_2$c.name,
			type: "if",
			source: "(104:16) {#if $spotifyConnection}",
			ctx
		});

		return block;
	}

	// (107:20) {#if $spotifyConnection.provider_user_id}
	function create_if_block_4$b(ctx) {
		let t0;
		let t1_value = /*$spotifyConnection*/ ctx[2].provider_user_id + "";
		let t1;

		const block = {
			c: function create() {
				t0 = text("• User ID: ");
				t1 = text(t1_value);
			},
			m: function mount(target, anchor) {
				insert_dev(target, t0, anchor);
				insert_dev(target, t1, anchor);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*$spotifyConnection*/ 4 && t1_value !== (t1_value = /*$spotifyConnection*/ ctx[2].provider_user_id + "")) set_data_dev(t1, t1_value);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(t0);
					detach_dev(t1);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_4$b.name,
			type: "if",
			source: "(107:20) {#if $spotifyConnection.provider_user_id}",
			ctx
		});

		return block;
	}

	// (111:18) {#if $spotifyConnection.scopes.length > 0}
	function create_if_block_3$b(ctx) {
		let p;
		let t0;
		let t1_value = /*$spotifyConnection*/ ctx[2].scopes.join(', ') + "";
		let t1;

		const block = {
			c: function create() {
				p = element("p");
				t0 = text("Permissions: ");
				t1 = text(t1_value);
				attr_dev(p, "class", "text-xs text-gray-400 mt-1");
				add_location(p, file$h, 125, 20, 4849);
			},
			m: function mount(target, anchor) {
				insert_dev(target, p, anchor);
				append_dev(p, t0);
				append_dev(p, t1);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*$spotifyConnection*/ 4 && t1_value !== (t1_value = /*$spotifyConnection*/ ctx[2].scopes.join(', ') + "")) set_data_dev(t1, t1_value);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(p);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_3$b.name,
			type: "if",
			source: "(111:18) {#if $spotifyConnection.scopes.length > 0}",
			ctx
		});

		return block;
	}

	// (139:12) {:else}
	function create_else_block$f(ctx) {
		let button;
		let mounted;
		let dispose;

		function select_block_type_2(ctx, dirty) {
			if (/*isConnecting*/ ctx[0]) return create_if_block_1$f;
			return create_else_block_1$8;
		}

		let current_block_type = select_block_type_2(ctx);
		let if_block = current_block_type(ctx);

		const block = {
			c: function create() {
				button = element("button");
				if_block.c();
				button.disabled = /*isConnecting*/ ctx[0];
				attr_dev(button, "class", "inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed");
				add_location(button, file$h, 153, 14, 6150);
			},
			m: function mount(target, anchor) {
				insert_dev(target, button, anchor);
				if_block.m(button, null);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*connectSpotify*/ ctx[3], false, false, false, false);
					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (current_block_type !== (current_block_type = select_block_type_2(ctx))) {
					if_block.d(1);
					if_block = current_block_type(ctx);

					if (if_block) {
						if_block.c();
						if_block.m(button, null);
					}
				}

				if (dirty & /*isConnecting*/ 1) {
					prop_dev(button, "disabled", /*isConnecting*/ ctx[0]);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(button);
				}

				if_block.d();
				mounted = false;
				dispose();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block$f.name,
			type: "else",
			source: "(139:12) {:else}",
			ctx
		});

		return block;
	}

	// (126:12) {#if $spotifyConnection}
	function create_if_block$g(ctx) {
		let button0;
		let t1;
		let button1;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				button0 = element("button");
				button0.textContent = "Check Health";
				t1 = space();
				button1 = element("button");
				button1.textContent = "Disconnect";
				attr_dev(button0, "class", "inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500");
				add_location(button0, file$h, 140, 14, 5381);
				attr_dev(button1, "class", "inline-flex items-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-red-700 bg-red-100 hover:bg-red-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500");
				add_location(button1, file$h, 146, 14, 5758);
			},
			m: function mount(target, anchor) {
				insert_dev(target, button0, anchor);
				insert_dev(target, t1, anchor);
				insert_dev(target, button1, anchor);

				if (!mounted) {
					dispose = [
						listen_dev(button0, "click", /*checkHealth*/ ctx[5], false, false, false, false),
						listen_dev(button1, "click", /*disconnectSpotify*/ ctx[4], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(button0);
					detach_dev(t1);
					detach_dev(button1);
				}

				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block$g.name,
			type: "if",
			source: "(126:12) {#if $spotifyConnection}",
			ctx
		});

		return block;
	}

	// (151:16) {:else}
	function create_else_block_1$8(ctx) {
		let t;

		const block = {
			c: function create() {
				t = text("Connect");
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
			id: create_else_block_1$8.name,
			type: "else",
			source: "(151:16) {:else}",
			ctx
		});

		return block;
	}

	// (145:16) {#if isConnecting}
	function create_if_block_1$f(ctx) {
		let svg;
		let circle;
		let path;
		let t;

		const block = {
			c: function create() {
				svg = svg_element("svg");
				circle = svg_element("circle");
				path = svg_element("path");
				t = text("\n                  Connecting...");
				attr_dev(circle, "class", "opacity-25");
				attr_dev(circle, "cx", "12");
				attr_dev(circle, "cy", "12");
				attr_dev(circle, "r", "10");
				attr_dev(circle, "stroke", "currentColor");
				attr_dev(circle, "stroke-width", "4");
				add_location(circle, file$h, 160, 20, 6746);
				attr_dev(path, "class", "opacity-75");
				attr_dev(path, "fill", "currentColor");
				attr_dev(path, "d", "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z");
				add_location(path, file$h, 161, 20, 6865);
				attr_dev(svg, "class", "animate-spin -ml-1 mr-2 h-4 w-4 text-white");
				attr_dev(svg, "xmlns", "http://www.w3.org/2000/svg");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				add_location(svg, file$h, 159, 18, 6602);
			},
			m: function mount(target, anchor) {
				insert_dev(target, svg, anchor);
				append_dev(svg, circle);
				append_dev(svg, path);
				insert_dev(target, t, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(svg);
					detach_dev(t);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_1$f.name,
			type: "if",
			source: "(145:16) {#if isConnecting}",
			ctx
		});

		return block;
	}

	function create_fragment$h(ctx) {
		let div27;
		let div0;
		let h2;
		let t1;
		let p0;
		let t3;
		let t4;
		let div21;
		let ul0;
		let li0;
		let div8;
		let div6;
		let div2;
		let div1;
		let svg0;
		let path0;
		let t5;
		let div5;
		let div3;
		let p1;
		let t7;
		let t8;
		let div4;
		let t9;
		let div7;
		let t10;
		let li1;
		let div14;
		let div13;
		let div10;
		let div9;
		let svg1;
		let path1;
		let t11;
		let div12;
		let div11;
		let p2;
		let t13;
		let span0;
		let t15;
		let p3;
		let t17;
		let button0;
		let t19;
		let li2;
		let div20;
		let div19;
		let div16;
		let div15;
		let svg2;
		let path2;
		let t20;
		let div18;
		let div17;
		let p4;
		let t22;
		let span1;
		let t24;
		let p5;
		let t26;
		let button1;
		let t28;
		let div26;
		let div25;
		let div22;
		let svg3;
		let path3;
		let t29;
		let div24;
		let h3;
		let t31;
		let div23;
		let p6;
		let t33;
		let ul1;
		let li3;
		let t35;
		let li4;
		let t37;
		let li5;
		let if_block0 = /*error*/ ctx[1] && create_if_block_6$b(ctx);
		let if_block1 = /*$spotifyConnection*/ ctx[2] && create_if_block_5$b(ctx);

		function select_block_type(ctx, dirty) {
			if (/*$spotifyConnection*/ ctx[2]) return create_if_block_2$c;
			return create_else_block_2$4;
		}

		let current_block_type = select_block_type(ctx);
		let if_block2 = current_block_type(ctx);

		function select_block_type_1(ctx, dirty) {
			if (/*$spotifyConnection*/ ctx[2]) return create_if_block$g;
			return create_else_block$f;
		}

		let current_block_type_1 = select_block_type_1(ctx);
		let if_block3 = current_block_type_1(ctx);

		const block = {
			c: function create() {
				div27 = element("div");
				div0 = element("div");
				h2 = element("h2");
				h2.textContent = "Service Connections";
				t1 = space();
				p0 = element("p");
				p0.textContent = "Connect your streaming service accounts to manage your blocklist across platforms.";
				t3 = space();
				if (if_block0) if_block0.c();
				t4 = space();
				div21 = element("div");
				ul0 = element("ul");
				li0 = element("li");
				div8 = element("div");
				div6 = element("div");
				div2 = element("div");
				div1 = element("div");
				svg0 = svg_element("svg");
				path0 = svg_element("path");
				t5 = space();
				div5 = element("div");
				div3 = element("div");
				p1 = element("p");
				p1.textContent = "Spotify";
				t7 = space();
				if (if_block1) if_block1.c();
				t8 = space();
				div4 = element("div");
				if_block2.c();
				t9 = space();
				div7 = element("div");
				if_block3.c();
				t10 = space();
				li1 = element("li");
				div14 = element("div");
				div13 = element("div");
				div10 = element("div");
				div9 = element("div");
				svg1 = svg_element("svg");
				path1 = svg_element("path");
				t11 = space();
				div12 = element("div");
				div11 = element("div");
				p2 = element("p");
				p2.textContent = "Apple Music";
				t13 = space();
				span0 = element("span");
				span0.textContent = "Coming Soon";
				t15 = space();
				p3 = element("p");
				p3.textContent = "Apple Music integration will be available in a future update";
				t17 = space();
				button0 = element("button");
				button0.textContent = "Coming Soon";
				t19 = space();
				li2 = element("li");
				div20 = element("div");
				div19 = element("div");
				div16 = element("div");
				div15 = element("div");
				svg2 = svg_element("svg");
				path2 = svg_element("path");
				t20 = space();
				div18 = element("div");
				div17 = element("div");
				p4 = element("p");
				p4.textContent = "YouTube Music";
				t22 = space();
				span1 = element("span");
				span1.textContent = "Coming Soon";
				t24 = space();
				p5 = element("p");
				p5.textContent = "YouTube Music integration will be available in a future update";
				t26 = space();
				button1 = element("button");
				button1.textContent = "Coming Soon";
				t28 = space();
				div26 = element("div");
				div25 = element("div");
				div22 = element("div");
				svg3 = svg_element("svg");
				path3 = svg_element("path");
				t29 = space();
				div24 = element("div");
				h3 = element("h3");
				h3.textContent = "About Service Connections";
				t31 = space();
				div23 = element("div");
				p6 = element("p");
				p6.textContent = "Service connections allow you to apply your Do-Not-Play list across different streaming platforms. \n            Each connection is secured with OAuth 2.0 and only requests the minimum permissions needed to manage your blocklist.";
				t33 = space();
				ul1 = element("ul");
				li3 = element("li");
				li3.textContent = "Spotify: Full library management and playlist modification";
				t35 = space();
				li4 = element("li");
				li4.textContent = "Apple Music: Limited library access (coming soon)";
				t37 = space();
				li5 = element("li");
				li5.textContent = "YouTube Music: Browser extension support only (coming soon)";
				attr_dev(h2, "class", "text-2xl font-bold text-gray-900");
				add_location(h2, file$h, 73, 4, 1859);
				attr_dev(p0, "class", "mt-1 text-sm text-gray-600");
				add_location(p0, file$h, 74, 4, 1933);
				attr_dev(div0, "class", "mb-6");
				add_location(div0, file$h, 72, 2, 1836);
				attr_dev(path0, "d", "M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.42 1.56-.299.421-1.02.599-1.559.3z");
				add_location(path0, file$h, 103, 18, 3263);
				attr_dev(svg0, "class", "h-6 w-6 text-white");
				attr_dev(svg0, "fill", "currentColor");
				attr_dev(svg0, "viewBox", "0 0 24 24");
				add_location(svg0, file$h, 102, 16, 3172);
				attr_dev(div1, "class", "h-10 w-10 rounded-full bg-green-500 flex items-center justify-center");
				add_location(div1, file$h, 101, 14, 3073);
				attr_dev(div2, "class", "flex-shrink-0 h-10 w-10");
				add_location(div2, file$h, 100, 12, 3021);
				attr_dev(p1, "class", "text-sm font-medium text-gray-900");
				add_location(p1, file$h, 109, 16, 4017);
				attr_dev(div3, "class", "flex items-center");
				add_location(div3, file$h, 108, 14, 3969);
				attr_dev(div4, "class", "mt-1");
				add_location(div4, file$h, 116, 14, 4400);
				attr_dev(div5, "class", "ml-4");
				add_location(div5, file$h, 107, 12, 3936);
				attr_dev(div6, "class", "flex items-center");
				add_location(div6, file$h, 99, 10, 2977);
				attr_dev(div7, "class", "flex items-center space-x-2");
				add_location(div7, file$h, 138, 10, 5288);
				attr_dev(div8, "class", "px-4 py-4 flex items-center justify-between");
				add_location(div8, file$h, 98, 8, 2909);
				add_location(li0, file$h, 97, 6, 2896);
				attr_dev(path1, "d", "M23.997 6.124c0-.738-.065-1.47-.24-2.19-.317-1.31-1.062-2.31-2.18-3.043C21.003.517 20.373.285 19.7.164c-.517-.093-1.038-.135-1.564-.15-.04-.001-.08-.004-.12-.004H5.986c-.04 0-.08.003-.12.004-.526.015-1.047.057-1.564.15-.673.121-1.303.353-1.877.727C1.307 1.624.562 2.624.245 3.934.07 4.654.005 5.386.005 6.124v11.748c0 .738.065 1.47.24 2.19.317 1.31 1.062 2.31 2.18 3.043.574.374 1.204.606 1.877.727.517.093 1.038.135 1.564.15.04.001.08.004.12.004h12.014c.04 0 .08-.003.12-.004.526-.015 1.047-.057 1.564-.15.673-.121 1.303-.353 1.877-.727 1.118-.733 1.863-1.733 2.18-3.043.175-.72.24-1.452.24-2.19V6.124zM12.001 4.009c2.47 0 4.471 2.001 4.471 4.471s-2.001 4.471-4.471 4.471-4.471-2.001-4.471-4.471 2.001-4.471 4.471-4.471zm0 7.542c1.693 0 3.071-1.378 3.071-3.071s-1.378-3.071-3.071-3.071-3.071 1.378-3.071 3.071 1.378 3.071 3.071 3.071z");
				add_location(path1, file$h, 180, 18, 7674);
				attr_dev(svg1, "class", "h-6 w-6 text-white");
				attr_dev(svg1, "fill", "currentColor");
				attr_dev(svg1, "viewBox", "0 0 24 24");
				add_location(svg1, file$h, 179, 16, 7583);
				attr_dev(div9, "class", "h-10 w-10 rounded-full bg-gray-400 flex items-center justify-center");
				add_location(div9, file$h, 178, 14, 7485);
				attr_dev(div10, "class", "flex-shrink-0 h-10 w-10");
				add_location(div10, file$h, 177, 12, 7433);
				attr_dev(p2, "class", "text-sm font-medium text-gray-900");
				add_location(p2, file$h, 186, 16, 8678);
				attr_dev(span0, "class", "ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium text-gray-600 bg-gray-100");
				add_location(span0, file$h, 187, 16, 8755);
				attr_dev(div11, "class", "flex items-center");
				add_location(div11, file$h, 185, 14, 8630);
				attr_dev(p3, "class", "text-sm text-gray-500 mt-1");
				add_location(p3, file$h, 191, 14, 8962);
				attr_dev(div12, "class", "ml-4");
				add_location(div12, file$h, 184, 12, 8597);
				attr_dev(div13, "class", "flex items-center");
				add_location(div13, file$h, 176, 10, 7389);
				button0.disabled = true;
				attr_dev(button0, "class", "inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-400 bg-gray-100 cursor-not-allowed");
				add_location(button0, file$h, 197, 10, 9154);
				attr_dev(div14, "class", "px-4 py-4 flex items-center justify-between opacity-50");
				add_location(div14, file$h, 175, 8, 7310);
				add_location(li1, file$h, 174, 6, 7297);
				attr_dev(path2, "d", "M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z");
				add_location(path2, file$h, 213, 18, 9846);
				attr_dev(svg2, "class", "h-6 w-6 text-white");
				attr_dev(svg2, "fill", "currentColor");
				attr_dev(svg2, "viewBox", "0 0 24 24");
				add_location(svg2, file$h, 212, 16, 9755);
				attr_dev(div15, "class", "h-10 w-10 rounded-full bg-red-500 flex items-center justify-center");
				add_location(div15, file$h, 211, 14, 9658);
				attr_dev(div16, "class", "flex-shrink-0 h-10 w-10");
				add_location(div16, file$h, 210, 12, 9606);
				attr_dev(p4, "class", "text-sm font-medium text-gray-900");
				add_location(p4, file$h, 219, 16, 10363);
				attr_dev(span1, "class", "ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium text-gray-600 bg-gray-100");
				add_location(span1, file$h, 220, 16, 10442);
				attr_dev(div17, "class", "flex items-center");
				add_location(div17, file$h, 218, 14, 10315);
				attr_dev(p5, "class", "text-sm text-gray-500 mt-1");
				add_location(p5, file$h, 224, 14, 10649);
				attr_dev(div18, "class", "ml-4");
				add_location(div18, file$h, 217, 12, 10282);
				attr_dev(div19, "class", "flex items-center");
				add_location(div19, file$h, 209, 10, 9562);
				button1.disabled = true;
				attr_dev(button1, "class", "inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-400 bg-gray-100 cursor-not-allowed");
				add_location(button1, file$h, 230, 10, 10843);
				attr_dev(div20, "class", "px-4 py-4 flex items-center justify-between opacity-50");
				add_location(div20, file$h, 208, 8, 9483);
				add_location(li2, file$h, 207, 6, 9470);
				attr_dev(ul0, "class", "divide-y divide-gray-200");
				add_location(ul0, file$h, 95, 4, 2818);
				attr_dev(div21, "class", "bg-white shadow overflow-hidden sm:rounded-md");
				add_location(div21, file$h, 94, 2, 2754);
				attr_dev(path3, "fill-rule", "evenodd");
				attr_dev(path3, "d", "M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z");
				attr_dev(path3, "clip-rule", "evenodd");
				add_location(path3, file$h, 246, 10, 11377);
				attr_dev(svg3, "class", "h-5 w-5 text-blue-400");
				attr_dev(svg3, "viewBox", "0 0 20 20");
				attr_dev(svg3, "fill", "currentColor");
				add_location(svg3, file$h, 245, 8, 11291);
				attr_dev(div22, "class", "flex-shrink-0");
				add_location(div22, file$h, 244, 6, 11255);
				attr_dev(h3, "class", "text-sm font-medium text-blue-800");
				add_location(h3, file$h, 250, 8, 11620);
				add_location(p6, file$h, 254, 10, 11776);
				add_location(li3, file$h, 259, 12, 12108);
				add_location(li4, file$h, 260, 12, 12188);
				add_location(li5, file$h, 261, 12, 12259);
				attr_dev(ul1, "class", "list-disc list-inside mt-2 space-y-1");
				add_location(ul1, file$h, 258, 10, 12046);
				attr_dev(div23, "class", "mt-2 text-sm text-blue-700");
				add_location(div23, file$h, 253, 8, 11725);
				attr_dev(div24, "class", "ml-3");
				add_location(div24, file$h, 249, 6, 11593);
				attr_dev(div25, "class", "flex");
				add_location(div25, file$h, 243, 4, 11230);
				attr_dev(div26, "class", "mt-6 bg-blue-50 border border-blue-200 rounded-md p-4");
				add_location(div26, file$h, 242, 2, 11158);
				attr_dev(div27, "class", "px-4 py-6 sm:px-0");
				add_location(div27, file$h, 71, 0, 1802);
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				insert_dev(target, div27, anchor);
				append_dev(div27, div0);
				append_dev(div0, h2);
				append_dev(div0, t1);
				append_dev(div0, p0);
				append_dev(div27, t3);
				if (if_block0) if_block0.m(div27, null);
				append_dev(div27, t4);
				append_dev(div27, div21);
				append_dev(div21, ul0);
				append_dev(ul0, li0);
				append_dev(li0, div8);
				append_dev(div8, div6);
				append_dev(div6, div2);
				append_dev(div2, div1);
				append_dev(div1, svg0);
				append_dev(svg0, path0);
				append_dev(div6, t5);
				append_dev(div6, div5);
				append_dev(div5, div3);
				append_dev(div3, p1);
				append_dev(div3, t7);
				if (if_block1) if_block1.m(div3, null);
				append_dev(div5, t8);
				append_dev(div5, div4);
				if_block2.m(div4, null);
				append_dev(div8, t9);
				append_dev(div8, div7);
				if_block3.m(div7, null);
				append_dev(ul0, t10);
				append_dev(ul0, li1);
				append_dev(li1, div14);
				append_dev(div14, div13);
				append_dev(div13, div10);
				append_dev(div10, div9);
				append_dev(div9, svg1);
				append_dev(svg1, path1);
				append_dev(div13, t11);
				append_dev(div13, div12);
				append_dev(div12, div11);
				append_dev(div11, p2);
				append_dev(div11, t13);
				append_dev(div11, span0);
				append_dev(div12, t15);
				append_dev(div12, p3);
				append_dev(div14, t17);
				append_dev(div14, button0);
				append_dev(ul0, t19);
				append_dev(ul0, li2);
				append_dev(li2, div20);
				append_dev(div20, div19);
				append_dev(div19, div16);
				append_dev(div16, div15);
				append_dev(div15, svg2);
				append_dev(svg2, path2);
				append_dev(div19, t20);
				append_dev(div19, div18);
				append_dev(div18, div17);
				append_dev(div17, p4);
				append_dev(div17, t22);
				append_dev(div17, span1);
				append_dev(div18, t24);
				append_dev(div18, p5);
				append_dev(div20, t26);
				append_dev(div20, button1);
				append_dev(div27, t28);
				append_dev(div27, div26);
				append_dev(div26, div25);
				append_dev(div25, div22);
				append_dev(div22, svg3);
				append_dev(svg3, path3);
				append_dev(div25, t29);
				append_dev(div25, div24);
				append_dev(div24, h3);
				append_dev(div24, t31);
				append_dev(div24, div23);
				append_dev(div23, p6);
				append_dev(div23, t33);
				append_dev(div23, ul1);
				append_dev(ul1, li3);
				append_dev(ul1, t35);
				append_dev(ul1, li4);
				append_dev(ul1, t37);
				append_dev(ul1, li5);
			},
			p: function update(ctx, [dirty]) {
				if (/*error*/ ctx[1]) {
					if (if_block0) {
						if_block0.p(ctx, dirty);
					} else {
						if_block0 = create_if_block_6$b(ctx);
						if_block0.c();
						if_block0.m(div27, t4);
					}
				} else if (if_block0) {
					if_block0.d(1);
					if_block0 = null;
				}

				if (/*$spotifyConnection*/ ctx[2]) {
					if (if_block1) {
						if_block1.p(ctx, dirty);
					} else {
						if_block1 = create_if_block_5$b(ctx);
						if_block1.c();
						if_block1.m(div3, null);
					}
				} else if (if_block1) {
					if_block1.d(1);
					if_block1 = null;
				}

				if (current_block_type === (current_block_type = select_block_type(ctx)) && if_block2) {
					if_block2.p(ctx, dirty);
				} else {
					if_block2.d(1);
					if_block2 = current_block_type(ctx);

					if (if_block2) {
						if_block2.c();
						if_block2.m(div4, null);
					}
				}

				if (current_block_type_1 === (current_block_type_1 = select_block_type_1(ctx)) && if_block3) {
					if_block3.p(ctx, dirty);
				} else {
					if_block3.d(1);
					if_block3 = current_block_type_1(ctx);

					if (if_block3) {
						if_block3.c();
						if_block3.m(div7, null);
					}
				}
			},
			i: noop,
			o: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div27);
				}

				if (if_block0) if_block0.d();
				if (if_block1) if_block1.d();
				if_block2.d();
				if_block3.d();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_fragment$h.name,
			type: "component",
			source: "",
			ctx
		});

		return block;
	}

	function getStatusColor$2(status) {
		switch (status) {
			case 'active':
				return 'text-green-600 bg-green-100';
			case 'expired':
				return 'text-yellow-600 bg-yellow-100';
			case 'error':
				return 'text-red-600 bg-red-100';
			default:
				return 'text-gray-600 bg-gray-100';
		}
	}

	function formatDate$6(dateString) {
		return new Date(dateString).toLocaleDateString();
	}

	function instance$h($$self, $$props, $$invalidate) {
		let $spotifyConnection;
		validate_store(spotifyConnection, 'spotifyConnection');
		component_subscribe($$self, spotifyConnection, $$value => $$invalidate(2, $spotifyConnection = $$value));
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('ServiceConnections', slots, []);
		let isConnecting = false;
		let error = '';

		onMount(() => {
			// Handle OAuth callback if present
			const urlParams = new URLSearchParams(window.location.search);

			const code = urlParams.get('code');
			const state = urlParams.get('state');

			if (code && state) {
				handleSpotifyCallback(code, state);

				// Clean up URL
				window.history.replaceState({}, document.title, window.location.pathname);
			}
		});

		async function handleSpotifyCallback(code, state) {
			$$invalidate(0, isConnecting = true);
			$$invalidate(1, error = '');
			const result = await connectionActions.handleSpotifyCallback(code, state);

			if (!result.success) {
				$$invalidate(1, error = result.message || 'Failed to connect Spotify');
			}

			$$invalidate(0, isConnecting = false);
		}

		async function connectSpotify() {
			$$invalidate(0, isConnecting = true);
			$$invalidate(1, error = '');

			try {
				await connectionActions.initiateSpotifyAuth();
			} catch(err) {
				$$invalidate(1, error = 'Failed to initiate Spotify connection');
				$$invalidate(0, isConnecting = false);
			}
		}

		async function disconnectSpotify() {
			const result = await connectionActions.disconnectSpotify();

			if (!result.success) {
				$$invalidate(1, error = result.message || 'Failed to disconnect Spotify');
			}
		}

		async function checkHealth() {
			await connectionActions.checkSpotifyHealth();
		}

		const writable_props = [];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<ServiceConnections> was created with unknown prop '${key}'`);
		});

		$$self.$capture_state = () => ({
			onMount,
			connectionActions,
			spotifyConnection,
			isConnecting,
			error,
			handleSpotifyCallback,
			connectSpotify,
			disconnectSpotify,
			checkHealth,
			getStatusColor: getStatusColor$2,
			formatDate: formatDate$6,
			$spotifyConnection
		});

		$$self.$inject_state = $$props => {
			if ('isConnecting' in $$props) $$invalidate(0, isConnecting = $$props.isConnecting);
			if ('error' in $$props) $$invalidate(1, error = $$props.error);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		return [
			isConnecting,
			error,
			$spotifyConnection,
			connectSpotify,
			disconnectSpotify,
			checkHealth
		];
	}

	class ServiceConnections extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance$h, create_fragment$h, safe_not_equal, {});

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "ServiceConnections",
				options,
				id: create_fragment$h.name
			});
		}
	}

	/* src/lib/components/ArtistSearch.svelte generated by Svelte v4.2.20 */
	const file$g = "src/lib/components/ArtistSearch.svelte";

	function get_each_context$8(ctx, list, i) {
		const child_ctx = ctx.slice();
		child_ctx[17] = list[i];
		return child_ctx;
	}

	function get_each_context_1$4(ctx, list, i) {
		const child_ctx = ctx.slice();
		child_ctx[20] = list[i];
		return child_ctx;
	}

	function get_each_context_2(ctx, list, i) {
		const child_ctx = ctx.slice();
		child_ctx[17] = list[i];
		return child_ctx;
	}

	// (82:6) {#if selectedArtist}
	function create_if_block_9$2(ctx) {
		let button;
		let svg;
		let path;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				button = element("button");
				svg = svg_element("svg");
				path = svg_element("path");
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M6 18L18 6M6 6l12 12");
				add_location(path, file$g, 100, 12, 2775);
				attr_dev(svg, "class", "h-5 w-5 text-gray-400 hover:text-gray-600");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$g, 99, 10, 2653);
				attr_dev(button, "type", "button");
				attr_dev(button, "class", "absolute inset-y-0 right-0 pr-3 flex items-center");
				add_location(button, file$g, 94, 8, 2497);
			},
			m: function mount(target, anchor) {
				insert_dev(target, button, anchor);
				append_dev(button, svg);
				append_dev(svg, path);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*clearSelection*/ ctx[9], false, false, false, false);
					mounted = true;
				}
			},
			p: noop,
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
			id: create_if_block_9$2.name,
			type: "if",
			source: "(82:6) {#if selectedArtist}",
			ctx
		});

		return block;
	}

	// (96:4) {#if $dnpStore.searchResults.length > 0 && !selectedArtist}
	function create_if_block_6$a(ctx) {
		let div;
		let each_value_1 = ensure_array_like_dev(/*$dnpStore*/ ctx[6].searchResults);
		let each_blocks = [];

		for (let i = 0; i < each_value_1.length; i += 1) {
			each_blocks[i] = create_each_block_1$4(get_each_context_1$4(ctx, each_value_1, i));
		}

		const block = {
			c: function create() {
				div = element("div");

				for (let i = 0; i < each_blocks.length; i += 1) {
					each_blocks[i].c();
				}

				attr_dev(div, "class", "absolute z-10 mt-1 w-full bg-white shadow-lg max-h-60 rounded-md py-1 text-base ring-1 ring-black ring-opacity-5 overflow-auto focus:outline-none sm:text-sm");
				add_location(div, file$g, 108, 6, 3030);
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
				if (dirty & /*selectArtist, $dnpStore, getProviderBadges*/ 320) {
					each_value_1 = ensure_array_like_dev(/*$dnpStore*/ ctx[6].searchResults);
					let i;

					for (i = 0; i < each_value_1.length; i += 1) {
						const child_ctx = get_each_context_1$4(ctx, each_value_1, i);

						if (each_blocks[i]) {
							each_blocks[i].p(child_ctx, dirty);
						} else {
							each_blocks[i] = create_each_block_1$4(child_ctx);
							each_blocks[i].c();
							each_blocks[i].m(div, null);
						}
					}

					for (; i < each_blocks.length; i += 1) {
						each_blocks[i].d(1);
					}

					each_blocks.length = each_value_1.length;
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
			id: create_if_block_6$a.name,
			type: "if",
			source: "(96:4) {#if $dnpStore.searchResults.length > 0 && !selectedArtist}",
			ctx
		});

		return block;
	}

	// (112:16) {:else}
	function create_else_block_2$3(ctx) {
		let div;
		let svg;
		let path;

		const block = {
			c: function create() {
				div = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z");
				add_location(path, file$g, 126, 22, 4075);
				attr_dev(svg, "class", "h-4 w-4 text-gray-600");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$g, 125, 20, 3963);
				attr_dev(div, "class", "h-8 w-8 rounded-full bg-gray-300 flex items-center justify-center");
				add_location(div, file$g, 124, 18, 3863);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, svg);
				append_dev(svg, path);
			},
			p: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block_2$3.name,
			type: "else",
			source: "(112:16) {:else}",
			ctx
		});

		return block;
	}

	// (106:16) {#if artist.metadata.image}
	function create_if_block_8$6(ctx) {
		let img;
		let img_src_value;
		let img_alt_value;

		const block = {
			c: function create() {
				img = element("img");
				if (!src_url_equal(img.src, img_src_value = /*artist*/ ctx[20].metadata.image)) attr_dev(img, "src", img_src_value);
				attr_dev(img, "alt", img_alt_value = /*artist*/ ctx[20].canonical_name);
				attr_dev(img, "class", "h-8 w-8 rounded-full object-cover");
				add_location(img, file$g, 118, 18, 3637);
			},
			m: function mount(target, anchor) {
				insert_dev(target, img, anchor);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*$dnpStore*/ 64 && !src_url_equal(img.src, img_src_value = /*artist*/ ctx[20].metadata.image)) {
					attr_dev(img, "src", img_src_value);
				}

				if (dirty & /*$dnpStore*/ 64 && img_alt_value !== (img_alt_value = /*artist*/ ctx[20].canonical_name)) {
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
			id: create_if_block_8$6.name,
			type: "if",
			source: "(106:16) {#if artist.metadata.image}",
			ctx
		});

		return block;
	}

	// (123:18) {#if artist.metadata.genres && artist.metadata.genres.length > 0}
	function create_if_block_7$8(ctx) {
		let div;
		let t_value = /*artist*/ ctx[20].metadata.genres.slice(0, 2).join(', ') + "";
		let t;

		const block = {
			c: function create() {
				div = element("div");
				t = text(t_value);
				attr_dev(div, "class", "text-xs text-gray-500");
				add_location(div, file$g, 135, 20, 4555);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, t);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*$dnpStore*/ 64 && t_value !== (t_value = /*artist*/ ctx[20].metadata.genres.slice(0, 2).join(', ') + "")) set_data_dev(t, t_value);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_7$8.name,
			type: "if",
			source: "(123:18) {#if artist.metadata.genres && artist.metadata.genres.length > 0}",
			ctx
		});

		return block;
	}

	// (132:16) {#each getProviderBadges(artist) as badge}
	function create_each_block_2(ctx) {
		let span;
		let t0_value = /*badge*/ ctx[17].name + "";
		let t0;
		let t1;
		let span_class_value;

		const block = {
			c: function create() {
				span = element("span");
				t0 = text(t0_value);
				t1 = space();
				attr_dev(span, "class", span_class_value = "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium " + /*badge*/ ctx[17].color);
				add_location(span, file$g, 144, 18, 4891);
			},
			m: function mount(target, anchor) {
				insert_dev(target, span, anchor);
				append_dev(span, t0);
				append_dev(span, t1);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*$dnpStore*/ 64 && t0_value !== (t0_value = /*badge*/ ctx[17].name + "")) set_data_dev(t0, t0_value);

				if (dirty & /*$dnpStore*/ 64 && span_class_value !== (span_class_value = "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium " + /*badge*/ ctx[17].color)) {
					attr_dev(span, "class", span_class_value);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(span);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_each_block_2.name,
			type: "each",
			source: "(132:16) {#each getProviderBadges(artist) as badge}",
			ctx
		});

		return block;
	}

	// (98:8) {#each $dnpStore.searchResults as artist}
	function create_each_block_1$4(ctx) {
		let button;
		let div4;
		let div2;
		let t0;
		let div1;
		let div0;
		let t1_value = /*artist*/ ctx[20].canonical_name + "";
		let t1;
		let t2;
		let t3;
		let div3;
		let t4;
		let mounted;
		let dispose;

		function select_block_type(ctx, dirty) {
			if (/*artist*/ ctx[20].metadata.image) return create_if_block_8$6;
			return create_else_block_2$3;
		}

		let current_block_type = select_block_type(ctx);
		let if_block0 = current_block_type(ctx);
		let if_block1 = /*artist*/ ctx[20].metadata.genres && /*artist*/ ctx[20].metadata.genres.length > 0 && create_if_block_7$8(ctx);
		let each_value_2 = ensure_array_like_dev(getProviderBadges$2(/*artist*/ ctx[20]));
		let each_blocks = [];

		for (let i = 0; i < each_value_2.length; i += 1) {
			each_blocks[i] = create_each_block_2(get_each_context_2(ctx, each_value_2, i));
		}

		function click_handler() {
			return /*click_handler*/ ctx[13](/*artist*/ ctx[20]);
		}

		const block = {
			c: function create() {
				button = element("button");
				div4 = element("div");
				div2 = element("div");
				if_block0.c();
				t0 = space();
				div1 = element("div");
				div0 = element("div");
				t1 = text(t1_value);
				t2 = space();
				if (if_block1) if_block1.c();
				t3 = space();
				div3 = element("div");

				for (let i = 0; i < each_blocks.length; i += 1) {
					each_blocks[i].c();
				}

				t4 = space();
				attr_dev(div0, "class", "text-sm font-medium text-gray-900");
				add_location(div0, file$g, 131, 18, 4334);
				add_location(div1, file$g, 130, 16, 4310);
				attr_dev(div2, "class", "flex items-center space-x-3");
				add_location(div2, file$g, 116, 14, 3533);
				attr_dev(div3, "class", "flex space-x-1");
				add_location(div3, file$g, 142, 14, 4785);
				attr_dev(div4, "class", "flex items-center justify-between");
				add_location(div4, file$g, 115, 12, 3471);
				attr_dev(button, "type", "button");
				attr_dev(button, "class", "w-full text-left px-4 py-2 hover:bg-gray-100 focus:bg-gray-100 focus:outline-none");
				add_location(button, file$g, 110, 10, 3261);
			},
			m: function mount(target, anchor) {
				insert_dev(target, button, anchor);
				append_dev(button, div4);
				append_dev(div4, div2);
				if_block0.m(div2, null);
				append_dev(div2, t0);
				append_dev(div2, div1);
				append_dev(div1, div0);
				append_dev(div0, t1);
				append_dev(div1, t2);
				if (if_block1) if_block1.m(div1, null);
				append_dev(div4, t3);
				append_dev(div4, div3);

				for (let i = 0; i < each_blocks.length; i += 1) {
					if (each_blocks[i]) {
						each_blocks[i].m(div3, null);
					}
				}

				append_dev(button, t4);

				if (!mounted) {
					dispose = listen_dev(button, "click", click_handler, false, false, false, false);
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
						if_block0.m(div2, t0);
					}
				}

				if (dirty & /*$dnpStore*/ 64 && t1_value !== (t1_value = /*artist*/ ctx[20].canonical_name + "")) set_data_dev(t1, t1_value);

				if (/*artist*/ ctx[20].metadata.genres && /*artist*/ ctx[20].metadata.genres.length > 0) {
					if (if_block1) {
						if_block1.p(ctx, dirty);
					} else {
						if_block1 = create_if_block_7$8(ctx);
						if_block1.c();
						if_block1.m(div1, null);
					}
				} else if (if_block1) {
					if_block1.d(1);
					if_block1 = null;
				}

				if (dirty & /*getProviderBadges, $dnpStore*/ 64) {
					each_value_2 = ensure_array_like_dev(getProviderBadges$2(/*artist*/ ctx[20]));
					let i;

					for (i = 0; i < each_value_2.length; i += 1) {
						const child_ctx = get_each_context_2(ctx, each_value_2, i);

						if (each_blocks[i]) {
							each_blocks[i].p(child_ctx, dirty);
						} else {
							each_blocks[i] = create_each_block_2(child_ctx);
							each_blocks[i].c();
							each_blocks[i].m(div3, null);
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
					detach_dev(button);
				}

				if_block0.d();
				if (if_block1) if_block1.d();
				destroy_each(each_blocks, detaching);
				mounted = false;
				dispose();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_each_block_1$4.name,
			type: "each",
			source: "(98:8) {#each $dnpStore.searchResults as artist}",
			ctx
		});

		return block;
	}

	// (144:4) {#if $dnpStore.isSearching}
	function create_if_block_5$a(ctx) {
		let div;
		let svg;
		let circle;
		let path;
		let t0;
		let p;

		const block = {
			c: function create() {
				div = element("div");
				svg = svg_element("svg");
				circle = svg_element("circle");
				path = svg_element("path");
				t0 = space();
				p = element("p");
				p.textContent = "Searching...";
				attr_dev(circle, "class", "opacity-25");
				attr_dev(circle, "cx", "12");
				attr_dev(circle, "cy", "12");
				attr_dev(circle, "r", "10");
				attr_dev(circle, "stroke", "currentColor");
				attr_dev(circle, "stroke-width", "4");
				add_location(circle, file$g, 158, 10, 5400);
				attr_dev(path, "class", "opacity-75");
				attr_dev(path, "fill", "currentColor");
				attr_dev(path, "d", "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z");
				add_location(path, file$g, 159, 10, 5509);
				attr_dev(svg, "class", "animate-spin mx-auto h-5 w-5 text-gray-400");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				add_location(svg, file$g, 157, 8, 5301);
				attr_dev(p, "class", "text-sm text-gray-500 mt-1");
				add_location(p, file$g, 161, 8, 5701);
				attr_dev(div, "class", "absolute z-10 mt-1 w-full bg-white shadow-lg rounded-md py-2 text-center");
				add_location(div, file$g, 156, 6, 5206);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, svg);
				append_dev(svg, circle);
				append_dev(svg, path);
				append_dev(div, t0);
				append_dev(div, p);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_5$a.name,
			type: "if",
			source: "(144:4) {#if $dnpStore.isSearching}",
			ctx
		});

		return block;
	}

	// (156:2) {#if selectedArtist}
	function create_if_block_2$b(ctx) {
		let div4;
		let h4;
		let t1;
		let div3;
		let t2;
		let div2;
		let div0;
		let t3_value = /*selectedArtist*/ ctx[1].canonical_name + "";
		let t3;
		let t4;
		let t5;
		let div1;

		function select_block_type_1(ctx, dirty) {
			if (/*selectedArtist*/ ctx[1].metadata.image) return create_if_block_4$a;
			return create_else_block_1$7;
		}

		let current_block_type = select_block_type_1(ctx);
		let if_block0 = current_block_type(ctx);
		let if_block1 = /*selectedArtist*/ ctx[1].metadata.genres && /*selectedArtist*/ ctx[1].metadata.genres.length > 0 && create_if_block_3$a(ctx);
		let each_value = ensure_array_like_dev(getProviderBadges$2(/*selectedArtist*/ ctx[1]));
		let each_blocks = [];

		for (let i = 0; i < each_value.length; i += 1) {
			each_blocks[i] = create_each_block$8(get_each_context$8(ctx, each_value, i));
		}

		const block = {
			c: function create() {
				div4 = element("div");
				h4 = element("h4");
				h4.textContent = "Selected Artist";
				t1 = space();
				div3 = element("div");
				if_block0.c();
				t2 = space();
				div2 = element("div");
				div0 = element("div");
				t3 = text(t3_value);
				t4 = space();
				if (if_block1) if_block1.c();
				t5 = space();
				div1 = element("div");

				for (let i = 0; i < each_blocks.length; i += 1) {
					each_blocks[i].c();
				}

				attr_dev(h4, "class", "text-sm font-medium text-gray-900 mb-2");
				add_location(h4, file$g, 169, 6, 5897);
				attr_dev(div0, "class", "text-sm font-medium text-gray-900");
				add_location(div0, file$g, 185, 10, 6699);
				attr_dev(div1, "class", "flex space-x-1 mt-1");
				add_location(div1, file$g, 193, 10, 7051);
				attr_dev(div2, "class", "flex-1");
				add_location(div2, file$g, 184, 8, 6668);
				attr_dev(div3, "class", "flex items-center space-x-3");
				add_location(div3, file$g, 170, 6, 5975);
				attr_dev(div4, "class", "bg-gray-50 rounded-lg p-4");
				add_location(div4, file$g, 168, 4, 5851);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div4, anchor);
				append_dev(div4, h4);
				append_dev(div4, t1);
				append_dev(div4, div3);
				if_block0.m(div3, null);
				append_dev(div3, t2);
				append_dev(div3, div2);
				append_dev(div2, div0);
				append_dev(div0, t3);
				append_dev(div2, t4);
				if (if_block1) if_block1.m(div2, null);
				append_dev(div2, t5);
				append_dev(div2, div1);

				for (let i = 0; i < each_blocks.length; i += 1) {
					if (each_blocks[i]) {
						each_blocks[i].m(div1, null);
					}
				}
			},
			p: function update(ctx, dirty) {
				if (current_block_type === (current_block_type = select_block_type_1(ctx)) && if_block0) {
					if_block0.p(ctx, dirty);
				} else {
					if_block0.d(1);
					if_block0 = current_block_type(ctx);

					if (if_block0) {
						if_block0.c();
						if_block0.m(div3, t2);
					}
				}

				if (dirty & /*selectedArtist*/ 2 && t3_value !== (t3_value = /*selectedArtist*/ ctx[1].canonical_name + "")) set_data_dev(t3, t3_value);

				if (/*selectedArtist*/ ctx[1].metadata.genres && /*selectedArtist*/ ctx[1].metadata.genres.length > 0) {
					if (if_block1) {
						if_block1.p(ctx, dirty);
					} else {
						if_block1 = create_if_block_3$a(ctx);
						if_block1.c();
						if_block1.m(div2, t5);
					}
				} else if (if_block1) {
					if_block1.d(1);
					if_block1 = null;
				}

				if (dirty & /*getProviderBadges, selectedArtist*/ 2) {
					each_value = ensure_array_like_dev(getProviderBadges$2(/*selectedArtist*/ ctx[1]));
					let i;

					for (i = 0; i < each_value.length; i += 1) {
						const child_ctx = get_each_context$8(ctx, each_value, i);

						if (each_blocks[i]) {
							each_blocks[i].p(child_ctx, dirty);
						} else {
							each_blocks[i] = create_each_block$8(child_ctx);
							each_blocks[i].c();
							each_blocks[i].m(div1, null);
						}
					}

					for (; i < each_blocks.length; i += 1) {
						each_blocks[i].d(1);
					}

					each_blocks.length = each_value.length;
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div4);
				}

				if_block0.d();
				if (if_block1) if_block1.d();
				destroy_each(each_blocks, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_2$b.name,
			type: "if",
			source: "(156:2) {#if selectedArtist}",
			ctx
		});

		return block;
	}

	// (166:8) {:else}
	function create_else_block_1$7(ctx) {
		let div;
		let svg;
		let path;

		const block = {
			c: function create() {
				div = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z");
				add_location(path, file$g, 180, 14, 6465);
				attr_dev(svg, "class", "h-6 w-6 text-gray-600");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$g, 179, 12, 6361);
				attr_dev(div, "class", "h-12 w-12 rounded-full bg-gray-300 flex items-center justify-center");
				add_location(div, file$g, 178, 10, 6267);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, svg);
				append_dev(svg, path);
			},
			p: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block_1$7.name,
			type: "else",
			source: "(166:8) {:else}",
			ctx
		});

		return block;
	}

	// (160:8) {#if selectedArtist.metadata.image}
	function create_if_block_4$a(ctx) {
		let img;
		let img_src_value;
		let img_alt_value;

		const block = {
			c: function create() {
				img = element("img");
				if (!src_url_equal(img.src, img_src_value = /*selectedArtist*/ ctx[1].metadata.image)) attr_dev(img, "src", img_src_value);
				attr_dev(img, "alt", img_alt_value = /*selectedArtist*/ ctx[1].canonical_name);
				attr_dev(img, "class", "h-12 w-12 rounded-full object-cover");
				add_location(img, file$g, 172, 10, 6071);
			},
			m: function mount(target, anchor) {
				insert_dev(target, img, anchor);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*selectedArtist*/ 2 && !src_url_equal(img.src, img_src_value = /*selectedArtist*/ ctx[1].metadata.image)) {
					attr_dev(img, "src", img_src_value);
				}

				if (dirty & /*selectedArtist*/ 2 && img_alt_value !== (img_alt_value = /*selectedArtist*/ ctx[1].canonical_name)) {
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
			id: create_if_block_4$a.name,
			type: "if",
			source: "(160:8) {#if selectedArtist.metadata.image}",
			ctx
		});

		return block;
	}

	// (177:10) {#if selectedArtist.metadata.genres && selectedArtist.metadata.genres.length > 0}
	function create_if_block_3$a(ctx) {
		let div;
		let t_value = /*selectedArtist*/ ctx[1].metadata.genres.join(', ') + "";
		let t;

		const block = {
			c: function create() {
				div = element("div");
				t = text(t_value);
				attr_dev(div, "class", "text-xs text-gray-500");
				add_location(div, file$g, 189, 12, 6912);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, t);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*selectedArtist*/ 2 && t_value !== (t_value = /*selectedArtist*/ ctx[1].metadata.genres.join(', ') + "")) set_data_dev(t, t_value);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_3$a.name,
			type: "if",
			source: "(177:10) {#if selectedArtist.metadata.genres && selectedArtist.metadata.genres.length > 0}",
			ctx
		});

		return block;
	}

	// (183:12) {#each getProviderBadges(selectedArtist) as badge}
	function create_each_block$8(ctx) {
		let span;
		let t0_value = /*badge*/ ctx[17].name + "";
		let t0;
		let t1;
		let span_class_value;

		const block = {
			c: function create() {
				span = element("span");
				t0 = text(t0_value);
				t1 = space();
				attr_dev(span, "class", span_class_value = "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium " + /*badge*/ ctx[17].color);
				add_location(span, file$g, 195, 14, 7162);
			},
			m: function mount(target, anchor) {
				insert_dev(target, span, anchor);
				append_dev(span, t0);
				append_dev(span, t1);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*selectedArtist*/ 2 && t0_value !== (t0_value = /*badge*/ ctx[17].name + "")) set_data_dev(t0, t0_value);

				if (dirty & /*selectedArtist*/ 2 && span_class_value !== (span_class_value = "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium " + /*badge*/ ctx[17].color)) {
					attr_dev(span, "class", span_class_value);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(span);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_each_block$8.name,
			type: "each",
			source: "(183:12) {#each getProviderBadges(selectedArtist) as badge}",
			ctx
		});

		return block;
	}

	// (225:2) {#if error}
	function create_if_block_1$e(ctx) {
		let div;
		let t;

		const block = {
			c: function create() {
				div = element("div");
				t = text(/*error*/ ctx[5]);
				attr_dev(div, "class", "text-red-600 text-sm");
				add_location(div, file$g, 237, 4, 8487);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, t);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*error*/ 32) set_data_dev(t, /*error*/ ctx[5]);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_1$e.name,
			type: "if",
			source: "(225:2) {#if error}",
			ctx
		});

		return block;
	}

	// (251:6) {:else}
	function create_else_block$e(ctx) {
		let t;

		const block = {
			c: function create() {
				t = text("Add to DNP List");
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
			id: create_else_block$e.name,
			type: "else",
			source: "(251:6) {:else}",
			ctx
		});

		return block;
	}

	// (245:6) {#if isAdding}
	function create_if_block$f(ctx) {
		let svg;
		let circle;
		let path;
		let t;

		const block = {
			c: function create() {
				svg = svg_element("svg");
				circle = svg_element("circle");
				path = svg_element("path");
				t = text("\n        Adding...");
				attr_dev(circle, "class", "opacity-25");
				attr_dev(circle, "cx", "12");
				attr_dev(circle, "cy", "12");
				attr_dev(circle, "r", "10");
				attr_dev(circle, "stroke", "currentColor");
				attr_dev(circle, "stroke-width", "4");
				add_location(circle, file$g, 258, 10, 9436);
				attr_dev(path, "class", "opacity-75");
				attr_dev(path, "fill", "currentColor");
				attr_dev(path, "d", "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z");
				add_location(path, file$g, 259, 10, 9545);
				attr_dev(svg, "class", "animate-spin -ml-1 mr-2 h-4 w-4 text-white");
				attr_dev(svg, "xmlns", "http://www.w3.org/2000/svg");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				add_location(svg, file$g, 257, 8, 9302);
			},
			m: function mount(target, anchor) {
				insert_dev(target, svg, anchor);
				append_dev(svg, circle);
				append_dev(svg, path);
				insert_dev(target, t, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(svg);
					detach_dev(t);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block$f.name,
			type: "if",
			source: "(245:6) {#if isAdding}",
			ctx
		});

		return block;
	}

	function create_fragment$g(ctx) {
		let form;
		let div1;
		let label0;
		let t1;
		let div0;
		let input0;
		let t2;
		let t3;
		let t4;
		let t5;
		let t6;
		let div2;
		let label1;
		let t8;
		let input1;
		let t9;
		let p;
		let t11;
		let div3;
		let label2;
		let t13;
		let textarea;
		let t14;
		let t15;
		let div4;
		let button0;
		let t17;
		let button1;
		let button1_disabled_value;
		let mounted;
		let dispose;
		let if_block0 = /*selectedArtist*/ ctx[1] && create_if_block_9$2(ctx);
		let if_block1 = /*$dnpStore*/ ctx[6].searchResults.length > 0 && !/*selectedArtist*/ ctx[1] && create_if_block_6$a(ctx);
		let if_block2 = /*$dnpStore*/ ctx[6].isSearching && create_if_block_5$a(ctx);
		let if_block3 = /*selectedArtist*/ ctx[1] && create_if_block_2$b(ctx);
		let if_block4 = /*error*/ ctx[5] && create_if_block_1$e(ctx);

		function select_block_type_2(ctx, dirty) {
			if (/*isAdding*/ ctx[4]) return create_if_block$f;
			return create_else_block$e;
		}

		let current_block_type = select_block_type_2(ctx);
		let if_block5 = current_block_type(ctx);

		const block = {
			c: function create() {
				form = element("form");
				div1 = element("div");
				label0 = element("label");
				label0.textContent = "Artist Name";
				t1 = space();
				div0 = element("div");
				input0 = element("input");
				t2 = space();
				if (if_block0) if_block0.c();
				t3 = space();
				if (if_block1) if_block1.c();
				t4 = space();
				if (if_block2) if_block2.c();
				t5 = space();
				if (if_block3) if_block3.c();
				t6 = space();
				div2 = element("div");
				label1 = element("label");
				label1.textContent = "Tags (optional)";
				t8 = space();
				input1 = element("input");
				t9 = space();
				p = element("p");
				p.textContent = "Use tags to organize your DNP list. Separate multiple tags with commas.";
				t11 = space();
				div3 = element("div");
				label2 = element("label");
				label2.textContent = "Note (optional)";
				t13 = space();
				textarea = element("textarea");
				t14 = space();
				if (if_block4) if_block4.c();
				t15 = space();
				div4 = element("div");
				button0 = element("button");
				button0.textContent = "Cancel";
				t17 = space();
				button1 = element("button");
				if_block5.c();
				attr_dev(label0, "for", "artist-search");
				attr_dev(label0, "class", "block text-sm font-medium text-gray-700");
				add_location(label0, file$g, 81, 4, 1998);
				attr_dev(input0, "id", "artist-search");
				attr_dev(input0, "type", "text");
				attr_dev(input0, "placeholder", "Search for an artist...");
				attr_dev(input0, "class", "block w-full border border-gray-300 rounded-md px-3 py-2 placeholder-gray-500 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm");
				add_location(input0, file$g, 85, 6, 2143);
				attr_dev(div0, "class", "mt-1 relative");
				add_location(div0, file$g, 84, 4, 2109);
				attr_dev(div1, "class", "relative");
				add_location(div1, file$g, 80, 2, 1971);
				attr_dev(label1, "for", "tags");
				attr_dev(label1, "class", "block text-sm font-medium text-gray-700");
				add_location(label1, file$g, 207, 4, 7420);
				attr_dev(input1, "id", "tags");
				attr_dev(input1, "type", "text");
				attr_dev(input1, "placeholder", "e.g., controversial, personal, explicit (comma-separated)");
				attr_dev(input1, "class", "mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 placeholder-gray-500 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm");
				add_location(input1, file$g, 210, 4, 7526);
				attr_dev(p, "class", "mt-1 text-xs text-gray-500");
				add_location(p, file$g, 217, 4, 7853);
				add_location(div2, file$g, 206, 2, 7410);
				attr_dev(label2, "for", "note");
				attr_dev(label2, "class", "block text-sm font-medium text-gray-700");
				add_location(label2, file$g, 224, 4, 8017);
				attr_dev(textarea, "id", "note");
				attr_dev(textarea, "rows", "2");
				attr_dev(textarea, "placeholder", "Add a personal note about why you're blocking this artist...");
				attr_dev(textarea, "class", "mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 placeholder-gray-500 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm");
				add_location(textarea, file$g, 227, 4, 8123);
				add_location(div3, file$g, 223, 2, 8007);
				attr_dev(button0, "type", "button");
				attr_dev(button0, "class", "px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500");
				add_location(button0, file$g, 244, 4, 8628);
				attr_dev(button1, "type", "submit");
				button1.disabled = button1_disabled_value = /*isAdding*/ ctx[4] || !/*searchQuery*/ ctx[0].trim();
				attr_dev(button1, "class", "px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed");
				add_location(button1, file$g, 251, 4, 8932);
				attr_dev(div4, "class", "flex justify-end space-x-3");
				add_location(div4, file$g, 243, 2, 8583);
				attr_dev(form, "class", "space-y-4");
				add_location(form, file$g, 78, 0, 1879);
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				insert_dev(target, form, anchor);
				append_dev(form, div1);
				append_dev(div1, label0);
				append_dev(div1, t1);
				append_dev(div1, div0);
				append_dev(div0, input0);
				set_input_value(input0, /*searchQuery*/ ctx[0]);
				append_dev(div0, t2);
				if (if_block0) if_block0.m(div0, null);
				append_dev(div1, t3);
				if (if_block1) if_block1.m(div1, null);
				append_dev(div1, t4);
				if (if_block2) if_block2.m(div1, null);
				append_dev(form, t5);
				if (if_block3) if_block3.m(form, null);
				append_dev(form, t6);
				append_dev(form, div2);
				append_dev(div2, label1);
				append_dev(div2, t8);
				append_dev(div2, input1);
				set_input_value(input1, /*tags*/ ctx[2]);
				append_dev(div2, t9);
				append_dev(div2, p);
				append_dev(form, t11);
				append_dev(form, div3);
				append_dev(div3, label2);
				append_dev(div3, t13);
				append_dev(div3, textarea);
				set_input_value(textarea, /*note*/ ctx[3]);
				append_dev(form, t14);
				if (if_block4) if_block4.m(form, null);
				append_dev(form, t15);
				append_dev(form, div4);
				append_dev(div4, button0);
				append_dev(div4, t17);
				append_dev(div4, button1);
				if_block5.m(button1, null);

				if (!mounted) {
					dispose = [
						listen_dev(input0, "input", /*input0_input_handler*/ ctx[12]),
						listen_dev(input1, "input", /*input1_input_handler*/ ctx[14]),
						listen_dev(textarea, "input", /*textarea_input_handler*/ ctx[15]),
						listen_dev(button0, "click", /*click_handler_1*/ ctx[16], false, false, false, false),
						listen_dev(form, "submit", prevent_default(/*handleSubmit*/ ctx[10]), false, true, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, [dirty]) {
				if (dirty & /*searchQuery*/ 1 && input0.value !== /*searchQuery*/ ctx[0]) {
					set_input_value(input0, /*searchQuery*/ ctx[0]);
				}

				if (/*selectedArtist*/ ctx[1]) {
					if (if_block0) {
						if_block0.p(ctx, dirty);
					} else {
						if_block0 = create_if_block_9$2(ctx);
						if_block0.c();
						if_block0.m(div0, null);
					}
				} else if (if_block0) {
					if_block0.d(1);
					if_block0 = null;
				}

				if (/*$dnpStore*/ ctx[6].searchResults.length > 0 && !/*selectedArtist*/ ctx[1]) {
					if (if_block1) {
						if_block1.p(ctx, dirty);
					} else {
						if_block1 = create_if_block_6$a(ctx);
						if_block1.c();
						if_block1.m(div1, t4);
					}
				} else if (if_block1) {
					if_block1.d(1);
					if_block1 = null;
				}

				if (/*$dnpStore*/ ctx[6].isSearching) {
					if (if_block2) ; else {
						if_block2 = create_if_block_5$a(ctx);
						if_block2.c();
						if_block2.m(div1, null);
					}
				} else if (if_block2) {
					if_block2.d(1);
					if_block2 = null;
				}

				if (/*selectedArtist*/ ctx[1]) {
					if (if_block3) {
						if_block3.p(ctx, dirty);
					} else {
						if_block3 = create_if_block_2$b(ctx);
						if_block3.c();
						if_block3.m(form, t6);
					}
				} else if (if_block3) {
					if_block3.d(1);
					if_block3 = null;
				}

				if (dirty & /*tags*/ 4 && input1.value !== /*tags*/ ctx[2]) {
					set_input_value(input1, /*tags*/ ctx[2]);
				}

				if (dirty & /*note*/ 8) {
					set_input_value(textarea, /*note*/ ctx[3]);
				}

				if (/*error*/ ctx[5]) {
					if (if_block4) {
						if_block4.p(ctx, dirty);
					} else {
						if_block4 = create_if_block_1$e(ctx);
						if_block4.c();
						if_block4.m(form, t15);
					}
				} else if (if_block4) {
					if_block4.d(1);
					if_block4 = null;
				}

				if (current_block_type !== (current_block_type = select_block_type_2(ctx))) {
					if_block5.d(1);
					if_block5 = current_block_type(ctx);

					if (if_block5) {
						if_block5.c();
						if_block5.m(button1, null);
					}
				}

				if (dirty & /*isAdding, searchQuery*/ 17 && button1_disabled_value !== (button1_disabled_value = /*isAdding*/ ctx[4] || !/*searchQuery*/ ctx[0].trim())) {
					prop_dev(button1, "disabled", button1_disabled_value);
				}
			},
			i: noop,
			o: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(form);
				}

				if (if_block0) if_block0.d();
				if (if_block1) if_block1.d();
				if (if_block2) if_block2.d();
				if (if_block3) if_block3.d();
				if (if_block4) if_block4.d();
				if_block5.d();
				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_fragment$g.name,
			type: "component",
			source: "",
			ctx
		});

		return block;
	}

	function getProviderBadges$2(artist) {
		const badges = [];

		if (artist.external_ids.spotify) badges.push({
			name: 'Spotify',
			color: 'bg-green-100 text-green-800'
		});

		if (artist.external_ids.apple) badges.push({
			name: 'Apple',
			color: 'bg-gray-100 text-gray-800'
		});

		if (artist.external_ids.musicbrainz) badges.push({
			name: 'MusicBrainz',
			color: 'bg-blue-100 text-blue-800'
		});

		return badges;
	}

	function instance$g($$self, $$props, $$invalidate) {
		let $dnpStore;
		validate_store(dnpStore, 'dnpStore');
		component_subscribe($$self, dnpStore, $$value => $$invalidate(6, $dnpStore = $$value));
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('ArtistSearch', slots, []);
		const dispatch = createEventDispatcher();
		let searchQuery = '';
		let selectedArtist = null;
		let tags = '';
		let note = '';
		let isAdding = false;
		let error = '';
		let searchTimeout;

		function selectArtist(artist) {
			$$invalidate(1, selectedArtist = artist);
			$$invalidate(0, searchQuery = artist.canonical_name);
			dnpActions.clearSearch();
		}

		function clearSelection() {
			$$invalidate(1, selectedArtist = null);
			$$invalidate(0, searchQuery = '');
			dnpActions.clearSearch();
		}

		async function handleSubmit() {
			if (!searchQuery.trim()) {
				$$invalidate(5, error = 'Please enter an artist name');
				return;
			}

			$$invalidate(4, isAdding = true);
			$$invalidate(5, error = '');
			const tagArray = tags.split(',').map(t => t.trim()).filter(t => t);
			const result = await dnpActions.addArtist(searchQuery, tagArray, note.trim() || undefined);

			if (result.success) {
				// Reset form
				$$invalidate(0, searchQuery = '');

				$$invalidate(1, selectedArtist = null);
				$$invalidate(2, tags = '');
				$$invalidate(3, note = '');
				dispatch('artistAdded');
			} else {
				$$invalidate(5, error = result.message || 'Failed to add artist');
			}

			$$invalidate(4, isAdding = false);
		}

		const writable_props = [];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<ArtistSearch> was created with unknown prop '${key}'`);
		});

		function input0_input_handler() {
			searchQuery = this.value;
			$$invalidate(0, searchQuery);
		}

		const click_handler = artist => selectArtist(artist);

		function input1_input_handler() {
			tags = this.value;
			$$invalidate(2, tags);
		}

		function textarea_input_handler() {
			note = this.value;
			$$invalidate(3, note);
		}

		const click_handler_1 = () => dispatch('artistAdded');

		$$self.$capture_state = () => ({
			createEventDispatcher,
			dnpActions,
			dnpStore,
			dispatch,
			searchQuery,
			selectedArtist,
			tags,
			note,
			isAdding,
			error,
			searchTimeout,
			selectArtist,
			clearSelection,
			handleSubmit,
			getProviderBadges: getProviderBadges$2,
			$dnpStore
		});

		$$self.$inject_state = $$props => {
			if ('searchQuery' in $$props) $$invalidate(0, searchQuery = $$props.searchQuery);
			if ('selectedArtist' in $$props) $$invalidate(1, selectedArtist = $$props.selectedArtist);
			if ('tags' in $$props) $$invalidate(2, tags = $$props.tags);
			if ('note' in $$props) $$invalidate(3, note = $$props.note);
			if ('isAdding' in $$props) $$invalidate(4, isAdding = $$props.isAdding);
			if ('error' in $$props) $$invalidate(5, error = $$props.error);
			if ('searchTimeout' in $$props) $$invalidate(11, searchTimeout = $$props.searchTimeout);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		$$self.$$.update = () => {
			if ($$self.$$.dirty & /*searchTimeout, searchQuery*/ 2049) {
				{
					if (searchTimeout) clearTimeout(searchTimeout);

					$$invalidate(11, searchTimeout = setTimeout(
						() => {
							if (searchQuery.trim()) {
								dnpActions.searchArtists(searchQuery);
							} else {
								dnpActions.clearSearch();
							}
						},
						300
					));
				}
			}
		};

		return [
			searchQuery,
			selectedArtist,
			tags,
			note,
			isAdding,
			error,
			$dnpStore,
			dispatch,
			selectArtist,
			clearSelection,
			handleSubmit,
			searchTimeout,
			input0_input_handler,
			click_handler,
			input1_input_handler,
			textarea_input_handler,
			click_handler_1
		];
	}

	class ArtistSearch extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance$g, create_fragment$g, safe_not_equal, {});

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "ArtistSearch",
				options,
				id: create_fragment$g.name
			});
		}
	}

	/* src/lib/components/DnpEntry.svelte generated by Svelte v4.2.20 */
	const file$f = "src/lib/components/DnpEntry.svelte";

	function get_each_context$7(ctx, list, i) {
		const child_ctx = ctx.slice();
		child_ctx[16] = list[i];
		return child_ctx;
	}

	function get_each_context_1$3(ctx, list, i) {
		const child_ctx = ctx.slice();
		child_ctx[19] = list[i];
		return child_ctx;
	}

	// (77:8) {#if !isEditing}
	function create_if_block_8$5(ctx) {
		let button0;
		let t1;
		let button1;
		let t2_value = (/*isRemoving*/ ctx[6] ? 'Removing...' : 'Remove') + "";
		let t2;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				button0 = element("button");
				button0.textContent = "Edit";
				t1 = space();
				button1 = element("button");
				t2 = text(t2_value);
				attr_dev(button0, "class", "text-indigo-600 hover:text-indigo-900 text-sm");
				add_location(button0, file$f, 96, 10, 2562);
				button1.disabled = /*isRemoving*/ ctx[6];
				attr_dev(button1, "class", "text-red-600 hover:text-red-900 text-sm disabled:opacity-50");
				add_location(button1, file$f, 102, 10, 2728);
			},
			m: function mount(target, anchor) {
				insert_dev(target, button0, anchor);
				insert_dev(target, t1, anchor);
				insert_dev(target, button1, anchor);
				append_dev(button1, t2);

				if (!mounted) {
					dispose = [
						listen_dev(button0, "click", /*startEdit*/ ctx[9], false, false, false, false),
						listen_dev(button1, "click", /*removeArtist*/ ctx[12], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (dirty & /*isRemoving*/ 64 && t2_value !== (t2_value = (/*isRemoving*/ ctx[6] ? 'Removing...' : 'Remove') + "")) set_data_dev(t2, t2_value);

				if (dirty & /*isRemoving*/ 64) {
					prop_dev(button1, "disabled", /*isRemoving*/ ctx[6]);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(button0);
					detach_dev(t1);
					detach_dev(button1);
				}

				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_8$5.name,
			type: "if",
			source: "(77:8) {#if !isEditing}",
			ctx
		});

		return block;
	}

	// (111:6) {:else}
	function create_else_block$d(ctx) {
		let div;
		let svg;
		let path;

		const block = {
			c: function create() {
				div = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z");
				add_location(path, file$f, 132, 12, 3777);
				attr_dev(svg, "class", "h-6 w-6 sm:h-5 sm:w-5 text-gray-600");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$f, 131, 10, 3661);
				attr_dev(div, "class", "h-12 w-12 sm:h-10 sm:w-10 rounded-full bg-gray-300 flex items-center justify-center");
				add_location(div, file$f, 130, 8, 3553);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, svg);
				append_dev(svg, path);
			},
			p: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block$d.name,
			type: "else",
			source: "(111:6) {:else}",
			ctx
		});

		return block;
	}

	// (105:6) {#if entry.artist.metadata.image}
	function create_if_block_7$7(ctx) {
		let img;
		let img_src_value;
		let img_alt_value;

		const block = {
			c: function create() {
				img = element("img");
				if (!src_url_equal(img.src, img_src_value = /*entry*/ ctx[0].artist.metadata.image)) attr_dev(img, "src", img_src_value);
				attr_dev(img, "alt", img_alt_value = /*entry*/ ctx[0].artist.canonical_name);
				attr_dev(img, "class", "h-12 w-12 sm:h-10 sm:w-10 rounded-full object-cover");
				add_location(img, file$f, 124, 8, 3357);
			},
			m: function mount(target, anchor) {
				insert_dev(target, img, anchor);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*entry*/ 1 && !src_url_equal(img.src, img_src_value = /*entry*/ ctx[0].artist.metadata.image)) {
					attr_dev(img, "src", img_src_value);
				}

				if (dirty & /*entry*/ 1 && img_alt_value !== (img_alt_value = /*entry*/ ctx[0].artist.canonical_name)) {
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
			id: create_if_block_7$7.name,
			type: "if",
			source: "(105:6) {#if entry.artist.metadata.image}",
			ctx
		});

		return block;
	}

	// (128:10) {#if entry.artist.metadata.genres && entry.artist.metadata.genres.length > 0}
	function create_if_block_6$9(ctx) {
		let p;
		let t_value = /*entry*/ ctx[0].artist.metadata.genres.slice(0, 2).join(', ') + "";
		let t;

		const block = {
			c: function create() {
				p = element("p");
				t = text(t_value);
				attr_dev(p, "class", "text-xs text-gray-500 truncate");
				add_location(p, file$f, 147, 12, 4387);
			},
			m: function mount(target, anchor) {
				insert_dev(target, p, anchor);
				append_dev(p, t);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*entry*/ 1 && t_value !== (t_value = /*entry*/ ctx[0].artist.metadata.genres.slice(0, 2).join(', ') + "")) set_data_dev(t, t_value);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(p);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_6$9.name,
			type: "if",
			source: "(128:10) {#if entry.artist.metadata.genres && entry.artist.metadata.genres.length > 0}",
			ctx
		});

		return block;
	}

	// (137:14) {#each getProviderBadges(entry.artist) as badge}
	function create_each_block_1$3(ctx) {
		let span;
		let t0_value = /*badge*/ ctx[19].name + "";
		let t0;
		let t1;
		let span_class_value;

		const block = {
			c: function create() {
				span = element("span");
				t0 = text(t0_value);
				t1 = space();
				attr_dev(span, "class", span_class_value = "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium " + /*badge*/ ctx[19].color);
				add_location(span, file$f, 156, 16, 4768);
			},
			m: function mount(target, anchor) {
				insert_dev(target, span, anchor);
				append_dev(span, t0);
				append_dev(span, t1);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*entry*/ 1 && t0_value !== (t0_value = /*badge*/ ctx[19].name + "")) set_data_dev(t0, t0_value);

				if (dirty & /*entry*/ 1 && span_class_value !== (span_class_value = "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium " + /*badge*/ ctx[19].color)) {
					attr_dev(span, "class", span_class_value);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(span);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_each_block_1$3.name,
			type: "each",
			source: "(137:14) {#each getProviderBadges(entry.artist) as badge}",
			ctx
		});

		return block;
	}

	// (153:10) {#if !isEditing}
	function create_if_block_5$9(ctx) {
		let button0;
		let t1;
		let button1;
		let t2_value = (/*isRemoving*/ ctx[6] ? 'Removing...' : 'Remove') + "";
		let t2;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				button0 = element("button");
				button0.textContent = "Edit";
				t1 = space();
				button1 = element("button");
				t2 = text(t2_value);
				attr_dev(button0, "class", "text-indigo-600 hover:text-indigo-900 text-sm whitespace-nowrap");
				add_location(button0, file$f, 172, 12, 5311);
				button1.disabled = /*isRemoving*/ ctx[6];
				attr_dev(button1, "class", "text-red-600 hover:text-red-900 text-sm disabled:opacity-50 whitespace-nowrap");
				add_location(button1, file$f, 178, 12, 5507);
			},
			m: function mount(target, anchor) {
				insert_dev(target, button0, anchor);
				insert_dev(target, t1, anchor);
				insert_dev(target, button1, anchor);
				append_dev(button1, t2);

				if (!mounted) {
					dispose = [
						listen_dev(button0, "click", /*startEdit*/ ctx[9], false, false, false, false),
						listen_dev(button1, "click", /*removeArtist*/ ctx[12], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (dirty & /*isRemoving*/ 64 && t2_value !== (t2_value = (/*isRemoving*/ ctx[6] ? 'Removing...' : 'Remove') + "")) set_data_dev(t2, t2_value);

				if (dirty & /*isRemoving*/ 64) {
					prop_dev(button1, "disabled", /*isRemoving*/ ctx[6]);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(button0);
					detach_dev(t1);
					detach_dev(button1);
				}

				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_5$9.name,
			type: "if",
			source: "(153:10) {#if !isEditing}",
			ctx
		});

		return block;
	}

	// (172:6) {#if !isEditing}
	function create_if_block_2$a(ctx) {
		let div;
		let t;
		let if_block0 = /*entry*/ ctx[0].tags.length > 0 && create_if_block_4$9(ctx);
		let if_block1 = /*entry*/ ctx[0].note && create_if_block_3$9(ctx);

		const block = {
			c: function create() {
				div = element("div");
				if (if_block0) if_block0.c();
				t = space();
				if (if_block1) if_block1.c();
				attr_dev(div, "class", "mt-2");
				add_location(div, file$f, 191, 8, 5892);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				if (if_block0) if_block0.m(div, null);
				append_dev(div, t);
				if (if_block1) if_block1.m(div, null);
			},
			p: function update(ctx, dirty) {
				if (/*entry*/ ctx[0].tags.length > 0) {
					if (if_block0) {
						if_block0.p(ctx, dirty);
					} else {
						if_block0 = create_if_block_4$9(ctx);
						if_block0.c();
						if_block0.m(div, t);
					}
				} else if (if_block0) {
					if_block0.d(1);
					if_block0 = null;
				}

				if (/*entry*/ ctx[0].note) {
					if (if_block1) {
						if_block1.p(ctx, dirty);
					} else {
						if_block1 = create_if_block_3$9(ctx);
						if_block1.c();
						if_block1.m(div, null);
					}
				} else if (if_block1) {
					if_block1.d(1);
					if_block1 = null;
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}

				if (if_block0) if_block0.d();
				if (if_block1) if_block1.d();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_2$a.name,
			type: "if",
			source: "(172:6) {#if !isEditing}",
			ctx
		});

		return block;
	}

	// (174:10) {#if entry.tags.length > 0}
	function create_if_block_4$9(ctx) {
		let div;
		let each_value = ensure_array_like_dev(/*entry*/ ctx[0].tags);
		let each_blocks = [];

		for (let i = 0; i < each_value.length; i += 1) {
			each_blocks[i] = create_each_block$7(get_each_context$7(ctx, each_value, i));
		}

		const block = {
			c: function create() {
				div = element("div");

				for (let i = 0; i < each_blocks.length; i += 1) {
					each_blocks[i].c();
				}

				attr_dev(div, "class", "flex flex-wrap gap-1 mb-2");
				add_location(div, file$f, 193, 12, 5961);
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
				if (dirty & /*entry*/ 1) {
					each_value = ensure_array_like_dev(/*entry*/ ctx[0].tags);
					let i;

					for (i = 0; i < each_value.length; i += 1) {
						const child_ctx = get_each_context$7(ctx, each_value, i);

						if (each_blocks[i]) {
							each_blocks[i].p(child_ctx, dirty);
						} else {
							each_blocks[i] = create_each_block$7(child_ctx);
							each_blocks[i].c();
							each_blocks[i].m(div, null);
						}
					}

					for (; i < each_blocks.length; i += 1) {
						each_blocks[i].d(1);
					}

					each_blocks.length = each_value.length;
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
			id: create_if_block_4$9.name,
			type: "if",
			source: "(174:10) {#if entry.tags.length > 0}",
			ctx
		});

		return block;
	}

	// (176:14) {#each entry.tags as tag}
	function create_each_block$7(ctx) {
		let span;
		let t0_value = /*tag*/ ctx[16] + "";
		let t0;
		let t1;

		const block = {
			c: function create() {
				span = element("span");
				t0 = text(t0_value);
				t1 = space();
				attr_dev(span, "class", "inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800");
				add_location(span, file$f, 195, 16, 6057);
			},
			m: function mount(target, anchor) {
				insert_dev(target, span, anchor);
				append_dev(span, t0);
				append_dev(span, t1);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*entry*/ 1 && t0_value !== (t0_value = /*tag*/ ctx[16] + "")) set_data_dev(t0, t0_value);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(span);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_each_block$7.name,
			type: "each",
			source: "(176:14) {#each entry.tags as tag}",
			ctx
		});

		return block;
	}

	// (184:10) {#if entry.note}
	function create_if_block_3$9(ctx) {
		let p;
		let t0;
		let t1_value = /*entry*/ ctx[0].note + "";
		let t1;
		let t2;

		const block = {
			c: function create() {
				p = element("p");
				t0 = text("\"");
				t1 = text(t1_value);
				t2 = text("\"");
				attr_dev(p, "class", "text-sm text-gray-600 italic");
				add_location(p, file$f, 203, 12, 6323);
			},
			m: function mount(target, anchor) {
				insert_dev(target, p, anchor);
				append_dev(p, t0);
				append_dev(p, t1);
				append_dev(p, t2);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*entry*/ 1 && t1_value !== (t1_value = /*entry*/ ctx[0].note + "")) set_data_dev(t1, t1_value);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(p);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_3$9.name,
			type: "if",
			source: "(184:10) {#if entry.note}",
			ctx
		});

		return block;
	}

	// (193:6) {#if isEditing}
	function create_if_block$e(ctx) {
		let div4;
		let div2;
		let div0;
		let label0;
		let t0;
		let label0_for_value;
		let t1;
		let input;
		let input_id_value;
		let t2;
		let div1;
		let label1;
		let t3;
		let label1_for_value;
		let t4;
		let textarea;
		let textarea_id_value;
		let t5;
		let t6;
		let div3;
		let button0;
		let t8;
		let button1;
		let t9_value = (/*isUpdating*/ ctx[5] ? 'Saving...' : 'Save') + "";
		let t9;
		let mounted;
		let dispose;
		let if_block = /*error*/ ctx[7] && create_if_block_1$d(ctx);

		const block = {
			c: function create() {
				div4 = element("div");
				div2 = element("div");
				div0 = element("div");
				label0 = element("label");
				t0 = text("Tags");
				t1 = space();
				input = element("input");
				t2 = space();
				div1 = element("div");
				label1 = element("label");
				t3 = text("Note");
				t4 = space();
				textarea = element("textarea");
				t5 = space();
				if (if_block) if_block.c();
				t6 = space();
				div3 = element("div");
				button0 = element("button");
				button0.textContent = "Cancel";
				t8 = space();
				button1 = element("button");
				t9 = text(t9_value);
				attr_dev(label0, "for", label0_for_value = "edit-tags-" + /*entry*/ ctx[0].artist.id);
				attr_dev(label0, "class", "block text-xs font-medium text-gray-700");
				add_location(label0, file$f, 215, 14, 6640);
				attr_dev(input, "id", input_id_value = "edit-tags-" + /*entry*/ ctx[0].artist.id);
				attr_dev(input, "type", "text");
				attr_dev(input, "placeholder", "comma-separated tags");
				attr_dev(input, "class", "mt-1 block w-full border border-gray-300 rounded-md px-2 py-1 text-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500");
				add_location(input, file$f, 216, 14, 6756);
				add_location(div0, file$f, 214, 12, 6620);
				attr_dev(label1, "for", label1_for_value = "edit-note-" + /*entry*/ ctx[0].artist.id);
				attr_dev(label1, "class", "block text-xs font-medium text-gray-700");
				add_location(label1, file$f, 226, 14, 7191);
				attr_dev(textarea, "id", textarea_id_value = "edit-note-" + /*entry*/ ctx[0].artist.id);
				attr_dev(textarea, "rows", "2");
				attr_dev(textarea, "placeholder", "Personal note...");
				attr_dev(textarea, "class", "mt-1 block w-full border border-gray-300 rounded-md px-2 py-1 text-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500");
				add_location(textarea, file$f, 227, 14, 7307);
				attr_dev(div1, "class", "sm:col-span-2");
				add_location(div1, file$f, 225, 12, 7149);
				attr_dev(div2, "class", "grid grid-cols-1 sm:grid-cols-2 gap-3");
				add_location(div2, file$f, 213, 10, 6556);
				attr_dev(button0, "type", "button");
				attr_dev(button0, "class", "w-full sm:w-auto px-3 py-2 sm:py-1 border border-gray-300 rounded-md text-sm sm:text-xs font-medium text-gray-700 bg-white hover:bg-gray-50");
				add_location(button0, file$f, 242, 12, 7904);
				attr_dev(button1, "type", "button");
				button1.disabled = /*isUpdating*/ ctx[5];
				attr_dev(button1, "class", "w-full sm:w-auto px-3 py-2 sm:py-1 border border-transparent rounded-md text-sm sm:text-xs font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50");
				add_location(button1, file$f, 249, 12, 8207);
				attr_dev(div3, "class", "flex flex-col sm:flex-row justify-end space-y-2 sm:space-y-0 sm:space-x-2");
				add_location(div3, file$f, 241, 10, 7804);
				attr_dev(div4, "class", "mt-3 space-y-3 sm:mt-4");
				add_location(div4, file$f, 212, 8, 6509);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div4, anchor);
				append_dev(div4, div2);
				append_dev(div2, div0);
				append_dev(div0, label0);
				append_dev(label0, t0);
				append_dev(div0, t1);
				append_dev(div0, input);
				set_input_value(input, /*editTags*/ ctx[3]);
				append_dev(div2, t2);
				append_dev(div2, div1);
				append_dev(div1, label1);
				append_dev(label1, t3);
				append_dev(div1, t4);
				append_dev(div1, textarea);
				set_input_value(textarea, /*editNote*/ ctx[4]);
				append_dev(div4, t5);
				if (if_block) if_block.m(div4, null);
				append_dev(div4, t6);
				append_dev(div4, div3);
				append_dev(div3, button0);
				append_dev(div3, t8);
				append_dev(div3, button1);
				append_dev(button1, t9);

				if (!mounted) {
					dispose = [
						listen_dev(input, "input", /*input_input_handler*/ ctx[13]),
						listen_dev(textarea, "input", /*textarea_input_handler*/ ctx[14]),
						listen_dev(button0, "click", /*cancelEdit*/ ctx[10], false, false, false, false),
						listen_dev(button1, "click", /*saveEdit*/ ctx[11], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (dirty & /*entry*/ 1 && label0_for_value !== (label0_for_value = "edit-tags-" + /*entry*/ ctx[0].artist.id)) {
					attr_dev(label0, "for", label0_for_value);
				}

				if (dirty & /*entry*/ 1 && input_id_value !== (input_id_value = "edit-tags-" + /*entry*/ ctx[0].artist.id)) {
					attr_dev(input, "id", input_id_value);
				}

				if (dirty & /*editTags*/ 8 && input.value !== /*editTags*/ ctx[3]) {
					set_input_value(input, /*editTags*/ ctx[3]);
				}

				if (dirty & /*entry*/ 1 && label1_for_value !== (label1_for_value = "edit-note-" + /*entry*/ ctx[0].artist.id)) {
					attr_dev(label1, "for", label1_for_value);
				}

				if (dirty & /*entry*/ 1 && textarea_id_value !== (textarea_id_value = "edit-note-" + /*entry*/ ctx[0].artist.id)) {
					attr_dev(textarea, "id", textarea_id_value);
				}

				if (dirty & /*editNote*/ 16) {
					set_input_value(textarea, /*editNote*/ ctx[4]);
				}

				if (/*error*/ ctx[7]) {
					if (if_block) {
						if_block.p(ctx, dirty);
					} else {
						if_block = create_if_block_1$d(ctx);
						if_block.c();
						if_block.m(div4, t6);
					}
				} else if (if_block) {
					if_block.d(1);
					if_block = null;
				}

				if (dirty & /*isUpdating*/ 32 && t9_value !== (t9_value = (/*isUpdating*/ ctx[5] ? 'Saving...' : 'Save') + "")) set_data_dev(t9, t9_value);

				if (dirty & /*isUpdating*/ 32) {
					prop_dev(button1, "disabled", /*isUpdating*/ ctx[5]);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div4);
				}

				if (if_block) if_block.d();
				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block$e.name,
			type: "if",
			source: "(193:6) {#if isEditing}",
			ctx
		});

		return block;
	}

	// (219:10) {#if error}
	function create_if_block_1$d(ctx) {
		let p;
		let t;

		const block = {
			c: function create() {
				p = element("p");
				t = text(/*error*/ ctx[7]);
				attr_dev(p, "class", "text-xs text-red-600");
				add_location(p, file$f, 238, 12, 7733);
			},
			m: function mount(target, anchor) {
				insert_dev(target, p, anchor);
				append_dev(p, t);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*error*/ 128) set_data_dev(t, /*error*/ ctx[7]);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(p);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_1$d.name,
			type: "if",
			source: "(219:10) {#if error}",
			ctx
		});

		return block;
	}

	function create_fragment$f(ctx) {
		let li;
		let div9;
		let div1;
		let input0;
		let t0;
		let div0;
		let t1;
		let input1;
		let t2;
		let div2;
		let t3;
		let div8;
		let div7;
		let div5;
		let p;
		let t4_value = /*entry*/ ctx[0].artist.canonical_name + "";
		let t4;
		let t5;
		let t6;
		let div4;
		let div3;
		let t7;
		let span;
		let t8;
		let t9_value = formatDate$5(/*entry*/ ctx[0].created_at) + "";
		let t9;
		let t10;
		let div6;
		let t11;
		let t12;
		let li_class_value;
		let mounted;
		let dispose;
		let if_block0 = !/*isEditing*/ ctx[2] && create_if_block_8$5(ctx);

		function select_block_type(ctx, dirty) {
			if (/*entry*/ ctx[0].artist.metadata.image) return create_if_block_7$7;
			return create_else_block$d;
		}

		let current_block_type = select_block_type(ctx);
		let if_block1 = current_block_type(ctx);
		let if_block2 = /*entry*/ ctx[0].artist.metadata.genres && /*entry*/ ctx[0].artist.metadata.genres.length > 0 && create_if_block_6$9(ctx);
		let each_value_1 = ensure_array_like_dev(getProviderBadges$1(/*entry*/ ctx[0].artist));
		let each_blocks = [];

		for (let i = 0; i < each_value_1.length; i += 1) {
			each_blocks[i] = create_each_block_1$3(get_each_context_1$3(ctx, each_value_1, i));
		}

		let if_block3 = !/*isEditing*/ ctx[2] && create_if_block_5$9(ctx);
		let if_block4 = !/*isEditing*/ ctx[2] && create_if_block_2$a(ctx);
		let if_block5 = /*isEditing*/ ctx[2] && create_if_block$e(ctx);

		const block = {
			c: function create() {
				li = element("li");
				div9 = element("div");
				div1 = element("div");
				input0 = element("input");
				t0 = space();
				div0 = element("div");
				if (if_block0) if_block0.c();
				t1 = space();
				input1 = element("input");
				t2 = space();
				div2 = element("div");
				if_block1.c();
				t3 = space();
				div8 = element("div");
				div7 = element("div");
				div5 = element("div");
				p = element("p");
				t4 = text(t4_value);
				t5 = space();
				if (if_block2) if_block2.c();
				t6 = space();
				div4 = element("div");
				div3 = element("div");

				for (let i = 0; i < each_blocks.length; i += 1) {
					each_blocks[i].c();
				}

				t7 = space();
				span = element("span");
				t8 = text("Added ");
				t9 = text(t9_value);
				t10 = space();
				div6 = element("div");
				if (if_block3) if_block3.c();
				t11 = space();
				if (if_block4) if_block4.c();
				t12 = space();
				if (if_block5) if_block5.c();
				attr_dev(input0, "type", "checkbox");
				input0.checked = /*selected*/ ctx[1];
				attr_dev(input0, "class", "h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded");
				add_location(input0, file$f, 88, 6, 2293);
				attr_dev(div0, "class", "flex items-center space-x-2");
				add_location(div0, file$f, 94, 6, 2485);
				attr_dev(div1, "class", "flex items-center justify-between sm:hidden");
				add_location(div1, file$f, 87, 4, 2229);
				attr_dev(input1, "type", "checkbox");
				input1.checked = /*selected*/ ctx[1];
				attr_dev(input1, "class", "hidden sm:block h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded flex-shrink-0");
				add_location(input1, file$f, 114, 4, 3044);
				attr_dev(div2, "class", "flex-shrink-0");
				add_location(div2, file$f, 122, 4, 3281);
				attr_dev(p, "class", "text-sm font-medium text-gray-900 truncate");
				add_location(p, file$f, 142, 10, 4164);
				attr_dev(div3, "class", "flex flex-wrap gap-1");
				add_location(div3, file$f, 154, 12, 4654);
				attr_dev(span, "class", "text-xs text-gray-400 whitespace-nowrap");
				add_location(span, file$f, 163, 12, 5015);
				attr_dev(div4, "class", "flex flex-wrap items-center gap-2 mt-1");
				add_location(div4, file$f, 152, 10, 4552);
				attr_dev(div5, "class", "flex-1 min-w-0");
				add_location(div5, file$f, 141, 8, 4125);
				attr_dev(div6, "class", "hidden sm:flex items-center space-x-2 ml-4");
				add_location(div6, file$f, 170, 8, 5215);
				attr_dev(div7, "class", "flex flex-col sm:flex-row sm:items-center sm:justify-between");
				add_location(div7, file$f, 140, 6, 4042);
				attr_dev(div8, "class", "flex-1 min-w-0");
				add_location(div8, file$f, 139, 4, 4007);
				attr_dev(div9, "class", "flex flex-col sm:flex-row sm:items-center space-y-3 sm:space-y-0 sm:space-x-4");
				add_location(div9, file$f, 85, 2, 2087);

				attr_dev(li, "class", li_class_value = "px-4 py-4 sm:px-6 " + (/*selected*/ ctx[1]
				? 'bg-indigo-50'
				: 'hover:bg-gray-50'));

				add_location(li, file$f, 84, 0, 2005);
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				insert_dev(target, li, anchor);
				append_dev(li, div9);
				append_dev(div9, div1);
				append_dev(div1, input0);
				append_dev(div1, t0);
				append_dev(div1, div0);
				if (if_block0) if_block0.m(div0, null);
				append_dev(div9, t1);
				append_dev(div9, input1);
				append_dev(div9, t2);
				append_dev(div9, div2);
				if_block1.m(div2, null);
				append_dev(div9, t3);
				append_dev(div9, div8);
				append_dev(div8, div7);
				append_dev(div7, div5);
				append_dev(div5, p);
				append_dev(p, t4);
				append_dev(div5, t5);
				if (if_block2) if_block2.m(div5, null);
				append_dev(div5, t6);
				append_dev(div5, div4);
				append_dev(div4, div3);

				for (let i = 0; i < each_blocks.length; i += 1) {
					if (each_blocks[i]) {
						each_blocks[i].m(div3, null);
					}
				}

				append_dev(div4, t7);
				append_dev(div4, span);
				append_dev(span, t8);
				append_dev(span, t9);
				append_dev(div7, t10);
				append_dev(div7, div6);
				if (if_block3) if_block3.m(div6, null);
				append_dev(div8, t11);
				if (if_block4) if_block4.m(div8, null);
				append_dev(div8, t12);
				if (if_block5) if_block5.m(div8, null);

				if (!mounted) {
					dispose = [
						listen_dev(input0, "change", /*toggleSelect*/ ctx[8], false, false, false, false),
						listen_dev(input1, "change", /*toggleSelect*/ ctx[8], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, [dirty]) {
				if (dirty & /*selected*/ 2) {
					prop_dev(input0, "checked", /*selected*/ ctx[1]);
				}

				if (!/*isEditing*/ ctx[2]) {
					if (if_block0) {
						if_block0.p(ctx, dirty);
					} else {
						if_block0 = create_if_block_8$5(ctx);
						if_block0.c();
						if_block0.m(div0, null);
					}
				} else if (if_block0) {
					if_block0.d(1);
					if_block0 = null;
				}

				if (dirty & /*selected*/ 2) {
					prop_dev(input1, "checked", /*selected*/ ctx[1]);
				}

				if (current_block_type === (current_block_type = select_block_type(ctx)) && if_block1) {
					if_block1.p(ctx, dirty);
				} else {
					if_block1.d(1);
					if_block1 = current_block_type(ctx);

					if (if_block1) {
						if_block1.c();
						if_block1.m(div2, null);
					}
				}

				if (dirty & /*entry*/ 1 && t4_value !== (t4_value = /*entry*/ ctx[0].artist.canonical_name + "")) set_data_dev(t4, t4_value);

				if (/*entry*/ ctx[0].artist.metadata.genres && /*entry*/ ctx[0].artist.metadata.genres.length > 0) {
					if (if_block2) {
						if_block2.p(ctx, dirty);
					} else {
						if_block2 = create_if_block_6$9(ctx);
						if_block2.c();
						if_block2.m(div5, t6);
					}
				} else if (if_block2) {
					if_block2.d(1);
					if_block2 = null;
				}

				if (dirty & /*getProviderBadges, entry*/ 1) {
					each_value_1 = ensure_array_like_dev(getProviderBadges$1(/*entry*/ ctx[0].artist));
					let i;

					for (i = 0; i < each_value_1.length; i += 1) {
						const child_ctx = get_each_context_1$3(ctx, each_value_1, i);

						if (each_blocks[i]) {
							each_blocks[i].p(child_ctx, dirty);
						} else {
							each_blocks[i] = create_each_block_1$3(child_ctx);
							each_blocks[i].c();
							each_blocks[i].m(div3, null);
						}
					}

					for (; i < each_blocks.length; i += 1) {
						each_blocks[i].d(1);
					}

					each_blocks.length = each_value_1.length;
				}

				if (dirty & /*entry*/ 1 && t9_value !== (t9_value = formatDate$5(/*entry*/ ctx[0].created_at) + "")) set_data_dev(t9, t9_value);

				if (!/*isEditing*/ ctx[2]) {
					if (if_block3) {
						if_block3.p(ctx, dirty);
					} else {
						if_block3 = create_if_block_5$9(ctx);
						if_block3.c();
						if_block3.m(div6, null);
					}
				} else if (if_block3) {
					if_block3.d(1);
					if_block3 = null;
				}

				if (!/*isEditing*/ ctx[2]) {
					if (if_block4) {
						if_block4.p(ctx, dirty);
					} else {
						if_block4 = create_if_block_2$a(ctx);
						if_block4.c();
						if_block4.m(div8, t12);
					}
				} else if (if_block4) {
					if_block4.d(1);
					if_block4 = null;
				}

				if (/*isEditing*/ ctx[2]) {
					if (if_block5) {
						if_block5.p(ctx, dirty);
					} else {
						if_block5 = create_if_block$e(ctx);
						if_block5.c();
						if_block5.m(div8, null);
					}
				} else if (if_block5) {
					if_block5.d(1);
					if_block5 = null;
				}

				if (dirty & /*selected*/ 2 && li_class_value !== (li_class_value = "px-4 py-4 sm:px-6 " + (/*selected*/ ctx[1]
				? 'bg-indigo-50'
				: 'hover:bg-gray-50'))) {
					attr_dev(li, "class", li_class_value);
				}
			},
			i: noop,
			o: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(li);
				}

				if (if_block0) if_block0.d();
				if_block1.d();
				if (if_block2) if_block2.d();
				destroy_each(each_blocks, detaching);
				if (if_block3) if_block3.d();
				if (if_block4) if_block4.d();
				if (if_block5) if_block5.d();
				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_fragment$f.name,
			type: "component",
			source: "",
			ctx
		});

		return block;
	}

	function getProviderBadges$1(artist) {
		const badges = [];

		if (artist.external_ids.spotify) badges.push({
			name: 'Spotify',
			color: 'bg-green-100 text-green-800'
		});

		if (artist.external_ids.apple) badges.push({
			name: 'Apple',
			color: 'bg-gray-100 text-gray-800'
		});

		if (artist.external_ids.musicbrainz) badges.push({
			name: 'MusicBrainz',
			color: 'bg-blue-100 text-blue-800'
		});

		return badges;
	}

	function formatDate$5(dateString) {
		return new Date(dateString).toLocaleDateString();
	}

	function instance$f($$self, $$props, $$invalidate) {
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('DnpEntry', slots, []);
		const dispatch = createEventDispatcher();
		let { entry } = $$props;
		let { selected = false } = $$props;
		let isEditing = false;
		let editTags = entry.tags.join(', ');
		let editNote = entry.note || '';
		let isUpdating = false;
		let isRemoving = false;
		let error = '';

		function toggleSelect() {
			dispatch('toggleSelect');
		}

		function startEdit() {
			$$invalidate(2, isEditing = true);
			$$invalidate(3, editTags = entry.tags.join(', '));
			$$invalidate(4, editNote = entry.note || '');
			$$invalidate(7, error = '');
		}

		function cancelEdit() {
			$$invalidate(2, isEditing = false);
			$$invalidate(3, editTags = entry.tags.join(', '));
			$$invalidate(4, editNote = entry.note || '');
			$$invalidate(7, error = '');
		}

		async function saveEdit() {
			$$invalidate(5, isUpdating = true);
			$$invalidate(7, error = '');
			const tagArray = editTags.split(',').map(t => t.trim()).filter(t => t);
			const result = await dnpActions.updateEntry(entry.artist.id, tagArray, editNote.trim() || undefined);

			if (result.success) {
				$$invalidate(2, isEditing = false);
			} else {
				$$invalidate(7, error = result.message || 'Failed to update entry');
			}

			$$invalidate(5, isUpdating = false);
		}

		async function removeArtist() {
			if (!confirm(`Are you sure you want to remove "${entry.artist.canonical_name}" from your DNP list?`)) {
				return;
			}

			$$invalidate(6, isRemoving = true);
			const result = await dnpActions.removeArtist(entry.artist.id);

			if (!result.success) {
				$$invalidate(7, error = result.message || 'Failed to remove artist');
			}

			$$invalidate(6, isRemoving = false);
		}

		$$self.$$.on_mount.push(function () {
			if (entry === undefined && !('entry' in $$props || $$self.$$.bound[$$self.$$.props['entry']])) {
				console.warn("<DnpEntry> was created without expected prop 'entry'");
			}
		});

		const writable_props = ['entry', 'selected'];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<DnpEntry> was created with unknown prop '${key}'`);
		});

		function input_input_handler() {
			editTags = this.value;
			$$invalidate(3, editTags);
		}

		function textarea_input_handler() {
			editNote = this.value;
			$$invalidate(4, editNote);
		}

		$$self.$$set = $$props => {
			if ('entry' in $$props) $$invalidate(0, entry = $$props.entry);
			if ('selected' in $$props) $$invalidate(1, selected = $$props.selected);
		};

		$$self.$capture_state = () => ({
			createEventDispatcher,
			dnpActions,
			dispatch,
			entry,
			selected,
			isEditing,
			editTags,
			editNote,
			isUpdating,
			isRemoving,
			error,
			toggleSelect,
			startEdit,
			cancelEdit,
			saveEdit,
			removeArtist,
			getProviderBadges: getProviderBadges$1,
			formatDate: formatDate$5
		});

		$$self.$inject_state = $$props => {
			if ('entry' in $$props) $$invalidate(0, entry = $$props.entry);
			if ('selected' in $$props) $$invalidate(1, selected = $$props.selected);
			if ('isEditing' in $$props) $$invalidate(2, isEditing = $$props.isEditing);
			if ('editTags' in $$props) $$invalidate(3, editTags = $$props.editTags);
			if ('editNote' in $$props) $$invalidate(4, editNote = $$props.editNote);
			if ('isUpdating' in $$props) $$invalidate(5, isUpdating = $$props.isUpdating);
			if ('isRemoving' in $$props) $$invalidate(6, isRemoving = $$props.isRemoving);
			if ('error' in $$props) $$invalidate(7, error = $$props.error);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		return [
			entry,
			selected,
			isEditing,
			editTags,
			editNote,
			isUpdating,
			isRemoving,
			error,
			toggleSelect,
			startEdit,
			cancelEdit,
			saveEdit,
			removeArtist,
			input_input_handler,
			textarea_input_handler
		];
	}

	class DnpEntry extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance$f, create_fragment$f, safe_not_equal, { entry: 0, selected: 1 });

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "DnpEntry",
				options,
				id: create_fragment$f.name
			});
		}

		get entry() {
			throw new Error("<DnpEntry>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}

		set entry(value) {
			throw new Error("<DnpEntry>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}

		get selected() {
			throw new Error("<DnpEntry>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}

		set selected(value) {
			throw new Error("<DnpEntry>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}
	}

	/* src/lib/components/BulkActions.svelte generated by Svelte v4.2.20 */
	const file$e = "src/lib/components/BulkActions.svelte";

	function create_fragment$e(ctx) {
		let div3;
		let div2;
		let div0;
		let svg0;
		let path0;
		let t0;
		let p;
		let span;
		let t1;
		let t2;
		let t3_value = (/*selectedCount*/ ctx[0] === 1 ? 'artist' : 'artists') + "";
		let t3;
		let t4;
		let t5;
		let div1;
		let button0;
		let t7;
		let button1;
		let svg1;
		let path1;
		let t8;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				div3 = element("div");
				div2 = element("div");
				div0 = element("div");
				svg0 = svg_element("svg");
				path0 = svg_element("path");
				t0 = space();
				p = element("p");
				span = element("span");
				t1 = text(/*selectedCount*/ ctx[0]);
				t2 = space();
				t3 = text(t3_value);
				t4 = text(" selected");
				t5 = space();
				div1 = element("div");
				button0 = element("button");
				button0.textContent = "Clear selection";
				t7 = space();
				button1 = element("button");
				svg1 = svg_element("svg");
				path1 = svg_element("path");
				t8 = text("\n        Remove selected");
				attr_dev(path0, "stroke-linecap", "round");
				attr_dev(path0, "stroke-linejoin", "round");
				attr_dev(path0, "stroke-width", "2");
				attr_dev(path0, "d", "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z");
				add_location(path0, file$e, 20, 8, 600);
				attr_dev(svg0, "class", "h-5 w-5 text-indigo-400 flex-shrink-0");
				attr_dev(svg0, "fill", "none");
				attr_dev(svg0, "viewBox", "0 0 24 24");
				attr_dev(svg0, "stroke", "currentColor");
				add_location(svg0, file$e, 19, 6, 486);
				attr_dev(span, "class", "font-medium");
				add_location(span, file$e, 23, 8, 791);
				attr_dev(p, "class", "ml-3 text-sm text-indigo-800");
				add_location(p, file$e, 22, 6, 742);
				attr_dev(div0, "class", "flex items-center");
				add_location(div0, file$e, 18, 4, 448);
				attr_dev(button0, "class", "text-sm text-indigo-600 hover:text-indigo-500 text-center sm:text-left");
				add_location(button0, file$e, 29, 6, 1045);
				attr_dev(path1, "stroke-linecap", "round");
				attr_dev(path1, "stroke-linejoin", "round");
				attr_dev(path1, "stroke-width", "2");
				attr_dev(path1, "d", "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16");
				add_location(path1, file$e, 41, 10, 1645);
				attr_dev(svg1, "class", "-ml-0.5 mr-2 h-4 w-4");
				attr_dev(svg1, "fill", "none");
				attr_dev(svg1, "viewBox", "0 0 24 24");
				attr_dev(svg1, "stroke", "currentColor");
				add_location(svg1, file$e, 40, 8, 1546);
				attr_dev(button1, "class", "inline-flex items-center justify-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-red-700 bg-red-100 hover:bg-red-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500");
				add_location(button1, file$e, 36, 6, 1241);
				attr_dev(div1, "class", "flex flex-col sm:flex-row items-stretch sm:items-center space-y-2 sm:space-y-0 sm:space-x-3");
				add_location(div1, file$e, 28, 4, 933);
				attr_dev(div2, "class", "flex flex-col sm:flex-row sm:items-center sm:justify-between space-y-3 sm:space-y-0");
				add_location(div2, file$e, 17, 2, 346);
				attr_dev(div3, "class", "bg-indigo-50 border border-indigo-200 rounded-md p-4");
				add_location(div3, file$e, 16, 0, 277);
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				insert_dev(target, div3, anchor);
				append_dev(div3, div2);
				append_dev(div2, div0);
				append_dev(div0, svg0);
				append_dev(svg0, path0);
				append_dev(div0, t0);
				append_dev(div0, p);
				append_dev(p, span);
				append_dev(span, t1);
				append_dev(p, t2);
				append_dev(p, t3);
				append_dev(p, t4);
				append_dev(div2, t5);
				append_dev(div2, div1);
				append_dev(div1, button0);
				append_dev(div1, t7);
				append_dev(div1, button1);
				append_dev(button1, svg1);
				append_dev(svg1, path1);
				append_dev(button1, t8);

				if (!mounted) {
					dispose = [
						listen_dev(button0, "click", /*handleClearSelection*/ ctx[2], false, false, false, false),
						listen_dev(button1, "click", /*handleBulkDelete*/ ctx[1], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, [dirty]) {
				if (dirty & /*selectedCount*/ 1) set_data_dev(t1, /*selectedCount*/ ctx[0]);
				if (dirty & /*selectedCount*/ 1 && t3_value !== (t3_value = (/*selectedCount*/ ctx[0] === 1 ? 'artist' : 'artists') + "")) set_data_dev(t3, t3_value);
			},
			i: noop,
			o: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div3);
				}

				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_fragment$e.name,
			type: "component",
			source: "",
			ctx
		});

		return block;
	}

	function instance$e($$self, $$props, $$invalidate) {
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('BulkActions', slots, []);
		const dispatch = createEventDispatcher();
		let { selectedCount = 0 } = $$props;

		function handleBulkDelete() {
			dispatch('bulkDelete');
		}

		function handleClearSelection() {
			dispatch('clearSelection');
		}

		const writable_props = ['selectedCount'];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<BulkActions> was created with unknown prop '${key}'`);
		});

		$$self.$$set = $$props => {
			if ('selectedCount' in $$props) $$invalidate(0, selectedCount = $$props.selectedCount);
		};

		$$self.$capture_state = () => ({
			createEventDispatcher,
			dispatch,
			selectedCount,
			handleBulkDelete,
			handleClearSelection
		});

		$$self.$inject_state = $$props => {
			if ('selectedCount' in $$props) $$invalidate(0, selectedCount = $$props.selectedCount);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		return [selectedCount, handleBulkDelete, handleClearSelection];
	}

	class BulkActions extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance$e, create_fragment$e, safe_not_equal, { selectedCount: 0 });

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "BulkActions",
				options,
				id: create_fragment$e.name
			});
		}

		get selectedCount() {
			throw new Error("<BulkActions>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}

		set selectedCount(value) {
			throw new Error("<BulkActions>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}
	}

	/* src/lib/components/DnpManager.svelte generated by Svelte v4.2.20 */
	const file$d = "src/lib/components/DnpManager.svelte";

	function get_each_context$6(ctx, list, i) {
		const child_ctx = ctx.slice();
		child_ctx[19] = list[i];
		return child_ctx;
	}

	function get_each_context_1$2(ctx, list, i) {
		const child_ctx = ctx.slice();
		child_ctx[22] = list[i];
		return child_ctx;
	}

	// (75:2) {#if showAddForm}
	function create_if_block_8$4(ctx) {
		let div;
		let h3;
		let t1;
		let artistsearch;
		let current;
		artistsearch = new ArtistSearch({ $$inline: true });
		artistsearch.$on("artistAdded", /*handleArtistAdded*/ ctx[11]);

		const block = {
			c: function create() {
				div = element("div");
				h3 = element("h3");
				h3.textContent = "Add Artist to DNP List";
				t1 = space();
				create_component(artistsearch.$$.fragment);
				attr_dev(h3, "class", "text-lg font-medium text-gray-900 mb-4");
				add_location(h3, file$d, 87, 6, 2940);
				attr_dev(div, "class", "mb-6 bg-white shadow rounded-lg p-6");
				add_location(div, file$d, 86, 4, 2884);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, h3);
				append_dev(div, t1);
				mount_component(artistsearch, div, null);
				current = true;
			},
			p: noop,
			i: function intro(local) {
				if (current) return;
				transition_in(artistsearch.$$.fragment, local);
				current = true;
			},
			o: function outro(local) {
				transition_out(artistsearch.$$.fragment, local);
				current = false;
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}

				destroy_component(artistsearch);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_8$4.name,
			type: "if",
			source: "(75:2) {#if showAddForm}",
			ctx
		});

		return block;
	}

	// (111:10) {#each $dnpTags as tag}
	function create_each_block_1$2(ctx) {
		let option;
		let t_value = /*tag*/ ctx[22] + "";
		let t;
		let option_value_value;

		const block = {
			c: function create() {
				option = element("option");
				t = text(t_value);
				option.__value = option_value_value = /*tag*/ ctx[22];
				set_input_value(option, option.__value);
				add_location(option, file$d, 122, 12, 4612);
			},
			m: function mount(target, anchor) {
				insert_dev(target, option, anchor);
				append_dev(option, t);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*$dnpTags*/ 64 && t_value !== (t_value = /*tag*/ ctx[22] + "")) set_data_dev(t, t_value);

				if (dirty & /*$dnpTags*/ 64 && option_value_value !== (option_value_value = /*tag*/ ctx[22])) {
					prop_dev(option, "__value", option_value_value);
					set_input_value(option, option.__value);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(option);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_each_block_1$2.name,
			type: "each",
			source: "(111:10) {#each $dnpTags as tag}",
			ctx
		});

		return block;
	}

	// (120:2) {#if selectedEntries.size > 0}
	function create_if_block_7$6(ctx) {
		let div;
		let bulkactions;
		let current;

		bulkactions = new BulkActions({
				props: {
					selectedCount: /*selectedEntries*/ ctx[4].size
				},
				$$inline: true
			});

		bulkactions.$on("bulkDelete", /*handleBulkDelete*/ ctx[10]);
		bulkactions.$on("clearSelection", /*clearSelection*/ ctx[9]);

		const block = {
			c: function create() {
				div = element("div");
				create_component(bulkactions.$$.fragment);
				attr_dev(div, "class", "mb-4");
				add_location(div, file$d, 131, 4, 4778);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				mount_component(bulkactions, div, null);
				current = true;
			},
			p: function update(ctx, dirty) {
				const bulkactions_changes = {};
				if (dirty & /*selectedEntries*/ 16) bulkactions_changes.selectedCount = /*selectedEntries*/ ctx[4].size;
				bulkactions.$set(bulkactions_changes);
			},
			i: function intro(local) {
				if (current) return;
				transition_in(bulkactions.$$.fragment, local);
				current = true;
			},
			o: function outro(local) {
				transition_out(bulkactions.$$.fragment, local);
				current = false;
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}

				destroy_component(bulkactions);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_7$6.name,
			type: "if",
			source: "(120:2) {#if selectedEntries.size > 0}",
			ctx
		});

		return block;
	}

	// (180:4) {:else}
	function create_else_block_1$6(ctx) {
		let div2;
		let div1;
		let div0;
		let input;
		let input_checked_value;
		let t0;
		let label;
		let t1_value = /*filteredEntries*/ ctx[5].length + "";
		let t1;
		let t2;
		let t3_value = (/*filteredEntries*/ ctx[5].length !== 1 ? 's' : '') + "";
		let t3;
		let t4;
		let t5;
		let t6;
		let ul;
		let each_blocks = [];
		let each_1_lookup = new Map();
		let current;
		let mounted;
		let dispose;
		let if_block0 = /*selectedEntries*/ ctx[4].size > 0 && create_if_block_6$8(ctx);
		let if_block1 = (/*searchQuery*/ ctx[0] || /*selectedTag*/ ctx[1]) && create_if_block_5$8(ctx);
		let each_value = ensure_array_like_dev(/*filteredEntries*/ ctx[5]);
		const get_key = ctx => /*entry*/ ctx[19].artist.id;
		validate_each_keys(ctx, each_value, get_each_context$6, get_key);

		for (let i = 0; i < each_value.length; i += 1) {
			let child_ctx = get_each_context$6(ctx, each_value, i);
			let key = get_key(child_ctx);
			each_1_lookup.set(key, each_blocks[i] = create_each_block$6(key, child_ctx));
		}

		const block = {
			c: function create() {
				div2 = element("div");
				div1 = element("div");
				div0 = element("div");
				input = element("input");
				t0 = space();
				label = element("label");
				t1 = text(t1_value);
				t2 = text(" artist");
				t3 = text(t3_value);
				t4 = space();
				if (if_block0) if_block0.c();
				t5 = space();
				if (if_block1) if_block1.c();
				t6 = space();
				ul = element("ul");

				for (let i = 0; i < each_blocks.length; i += 1) {
					each_blocks[i].c();
				}

				attr_dev(input, "id", "select-all");
				attr_dev(input, "type", "checkbox");
				input.checked = input_checked_value = /*selectedEntries*/ ctx[4].size === /*filteredEntries*/ ctx[5].length && /*filteredEntries*/ ctx[5].length > 0;
				attr_dev(input, "class", "h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded");
				add_location(input, file$d, 194, 12, 8243);
				attr_dev(label, "for", "select-all");
				attr_dev(label, "class", "ml-3 text-sm text-gray-900");
				add_location(label, file$d, 201, 12, 8573);
				attr_dev(div0, "class", "flex items-center");
				add_location(div0, file$d, 193, 10, 8199);
				attr_dev(div1, "class", "flex items-center justify-between");
				add_location(div1, file$d, 192, 8, 8141);
				attr_dev(div2, "class", "px-4 py-3 bg-gray-50 border-b border-gray-200 sm:px-6");
				add_location(div2, file$d, 191, 6, 8065);
				attr_dev(ul, "class", "divide-y divide-gray-200");
				add_location(ul, file$d, 220, 6, 9207);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div2, anchor);
				append_dev(div2, div1);
				append_dev(div1, div0);
				append_dev(div0, input);
				append_dev(div0, t0);
				append_dev(div0, label);
				append_dev(label, t1);
				append_dev(label, t2);
				append_dev(label, t3);
				append_dev(label, t4);
				if (if_block0) if_block0.m(label, null);
				append_dev(div1, t5);
				if (if_block1) if_block1.m(div1, null);
				insert_dev(target, t6, anchor);
				insert_dev(target, ul, anchor);

				for (let i = 0; i < each_blocks.length; i += 1) {
					if (each_blocks[i]) {
						each_blocks[i].m(ul, null);
					}
				}

				current = true;

				if (!mounted) {
					dispose = listen_dev(input, "change", /*toggleSelectAll*/ ctx[7], false, false, false, false);
					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (!current || dirty & /*selectedEntries, filteredEntries*/ 48 && input_checked_value !== (input_checked_value = /*selectedEntries*/ ctx[4].size === /*filteredEntries*/ ctx[5].length && /*filteredEntries*/ ctx[5].length > 0)) {
					prop_dev(input, "checked", input_checked_value);
				}

				if ((!current || dirty & /*filteredEntries*/ 32) && t1_value !== (t1_value = /*filteredEntries*/ ctx[5].length + "")) set_data_dev(t1, t1_value);
				if ((!current || dirty & /*filteredEntries*/ 32) && t3_value !== (t3_value = (/*filteredEntries*/ ctx[5].length !== 1 ? 's' : '') + "")) set_data_dev(t3, t3_value);

				if (/*selectedEntries*/ ctx[4].size > 0) {
					if (if_block0) {
						if_block0.p(ctx, dirty);
					} else {
						if_block0 = create_if_block_6$8(ctx);
						if_block0.c();
						if_block0.m(label, null);
					}
				} else if (if_block0) {
					if_block0.d(1);
					if_block0 = null;
				}

				if (/*searchQuery*/ ctx[0] || /*selectedTag*/ ctx[1]) {
					if (if_block1) {
						if_block1.p(ctx, dirty);
					} else {
						if_block1 = create_if_block_5$8(ctx);
						if_block1.c();
						if_block1.m(div1, null);
					}
				} else if (if_block1) {
					if_block1.d(1);
					if_block1 = null;
				}

				if (dirty & /*filteredEntries, selectedEntries, toggleSelectEntry*/ 304) {
					each_value = ensure_array_like_dev(/*filteredEntries*/ ctx[5]);
					group_outros();
					validate_each_keys(ctx, each_value, get_each_context$6, get_key);
					each_blocks = update_keyed_each(each_blocks, dirty, get_key, 1, ctx, each_value, each_1_lookup, ul, outro_and_destroy_block, create_each_block$6, null, get_each_context$6);
					check_outros();
				}
			},
			i: function intro(local) {
				if (current) return;

				for (let i = 0; i < each_value.length; i += 1) {
					transition_in(each_blocks[i]);
				}

				current = true;
			},
			o: function outro(local) {
				for (let i = 0; i < each_blocks.length; i += 1) {
					transition_out(each_blocks[i]);
				}

				current = false;
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div2);
					detach_dev(t6);
					detach_dev(ul);
				}

				if (if_block0) if_block0.d();
				if (if_block1) if_block1.d();

				for (let i = 0; i < each_blocks.length; i += 1) {
					each_blocks[i].d();
				}

				mounted = false;
				dispose();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block_1$6.name,
			type: "else",
			source: "(180:4) {:else}",
			ctx
		});

		return block;
	}

	// (153:43) 
	function create_if_block_3$8(ctx) {
		let div;

		function select_block_type_1(ctx, dirty) {
			if (/*$dnpStore*/ ctx[2].entries.length === 0) return create_if_block_4$8;
			return create_else_block$c;
		}

		let current_block_type = select_block_type_1(ctx);
		let if_block = current_block_type(ctx);

		const block = {
			c: function create() {
				div = element("div");
				if_block.c();
				attr_dev(div, "class", "p-6 text-center");
				add_location(div, file$d, 164, 6, 6246);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				if_block.m(div, null);
			},
			p: function update(ctx, dirty) {
				if (current_block_type === (current_block_type = select_block_type_1(ctx)) && if_block) {
					if_block.p(ctx, dirty);
				} else {
					if_block.d(1);
					if_block = current_block_type(ctx);

					if (if_block) {
						if_block.c();
						if_block.m(div, null);
					}
				}
			},
			i: noop,
			o: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}

				if_block.d();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_3$8.name,
			type: "if",
			source: "(153:43) ",
			ctx
		});

		return block;
	}

	// (140:30) 
	function create_if_block_2$9(ctx) {
		let div;
		let svg;
		let path;
		let t0;
		let p;
		let t1_value = /*$dnpStore*/ ctx[2].error + "";
		let t1;
		let t2;
		let button;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				div = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				p = element("p");
				t1 = text(t1_value);
				t2 = space();
				button = element("button");
				button.textContent = "Try again";
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z");
				add_location(path, file$d, 153, 10, 5788);
				attr_dev(svg, "class", "mx-auto h-8 w-8 text-red-400");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$d, 152, 8, 5681);
				attr_dev(p, "class", "mt-2 text-sm text-red-600");
				add_location(p, file$d, 155, 8, 5938);
				attr_dev(button, "class", "mt-2 text-sm text-indigo-600 hover:text-indigo-500");
				add_location(button, file$d, 156, 8, 6005);
				attr_dev(div, "class", "p-6 text-center");
				add_location(div, file$d, 151, 6, 5643);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, svg);
				append_dev(svg, path);
				append_dev(div, t0);
				append_dev(div, p);
				append_dev(p, t1);
				append_dev(div, t2);
				append_dev(div, button);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*click_handler_1*/ ctx[15], false, false, false, false);
					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (dirty & /*$dnpStore*/ 4 && t1_value !== (t1_value = /*$dnpStore*/ ctx[2].error + "")) set_data_dev(t1, t1_value);
			},
			i: noop,
			o: noop,
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
			id: create_if_block_2$9.name,
			type: "if",
			source: "(140:30) ",
			ctx
		});

		return block;
	}

	// (132:4) {#if $dnpStore.isLoading}
	function create_if_block_1$c(ctx) {
		let div;
		let svg;
		let circle;
		let path;
		let t0;
		let p;

		const block = {
			c: function create() {
				div = element("div");
				svg = svg_element("svg");
				circle = svg_element("circle");
				path = svg_element("path");
				t0 = space();
				p = element("p");
				p.textContent = "Loading DNP list...";
				attr_dev(circle, "class", "opacity-25");
				attr_dev(circle, "cx", "12");
				attr_dev(circle, "cy", "12");
				attr_dev(circle, "r", "10");
				attr_dev(circle, "stroke", "currentColor");
				attr_dev(circle, "stroke-width", "4");
				add_location(circle, file$d, 145, 10, 5230);
				attr_dev(path, "class", "opacity-75");
				attr_dev(path, "fill", "currentColor");
				attr_dev(path, "d", "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z");
				add_location(path, file$d, 146, 10, 5339);
				attr_dev(svg, "class", "animate-spin mx-auto h-8 w-8 text-gray-400");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				add_location(svg, file$d, 144, 8, 5131);
				attr_dev(p, "class", "mt-2 text-sm text-gray-500");
				add_location(p, file$d, 148, 8, 5531);
				attr_dev(div, "class", "p-6 text-center");
				add_location(div, file$d, 143, 6, 5093);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, svg);
				append_dev(svg, circle);
				append_dev(svg, path);
				append_dev(div, t0);
				append_dev(div, p);
			},
			p: noop,
			i: noop,
			o: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_1$c.name,
			type: "if",
			source: "(132:4) {#if $dnpStore.isLoading}",
			ctx
		});

		return block;
	}

	// (193:14) {#if selectedEntries.size > 0}
	function create_if_block_6$8(ctx) {
		let t0;
		let t1_value = /*selectedEntries*/ ctx[4].size + "";
		let t1;
		let t2;

		const block = {
			c: function create() {
				t0 = text("(");
				t1 = text(t1_value);
				t2 = text(" selected)");
			},
			m: function mount(target, anchor) {
				insert_dev(target, t0, anchor);
				insert_dev(target, t1, anchor);
				insert_dev(target, t2, anchor);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*selectedEntries*/ 16 && t1_value !== (t1_value = /*selectedEntries*/ ctx[4].size + "")) set_data_dev(t1, t1_value);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(t0);
					detach_dev(t1);
					detach_dev(t2);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_6$8.name,
			type: "if",
			source: "(193:14) {#if selectedEntries.size > 0}",
			ctx
		});

		return block;
	}

	// (199:10) {#if searchQuery || selectedTag}
	function create_if_block_5$8(ctx) {
		let button;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				button = element("button");
				button.textContent = "Clear filters";
				attr_dev(button, "class", "text-sm text-indigo-600 hover:text-indigo-500");
				add_location(button, file$d, 210, 12, 8939);
			},
			m: function mount(target, anchor) {
				insert_dev(target, button, anchor);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*click_handler_3*/ ctx[17], false, false, false, false);
					mounted = true;
				}
			},
			p: noop,
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
			id: create_if_block_5$8.name,
			type: "if",
			source: "(199:10) {#if searchQuery || selectedTag}",
			ctx
		});

		return block;
	}

	// (211:8) {#each filteredEntries as entry (entry.artist.id)}
	function create_each_block$6(key_1, ctx) {
		let first;
		let dnpentry;
		let current;

		function toggleSelect_handler() {
			return /*toggleSelect_handler*/ ctx[18](/*entry*/ ctx[19]);
		}

		dnpentry = new DnpEntry({
				props: {
					entry: /*entry*/ ctx[19],
					selected: /*selectedEntries*/ ctx[4].has(/*entry*/ ctx[19].artist.id)
				},
				$$inline: true
			});

		dnpentry.$on("toggleSelect", toggleSelect_handler);

		const block = {
			key: key_1,
			first: null,
			c: function create() {
				first = empty();
				create_component(dnpentry.$$.fragment);
				this.first = first;
			},
			m: function mount(target, anchor) {
				insert_dev(target, first, anchor);
				mount_component(dnpentry, target, anchor);
				current = true;
			},
			p: function update(new_ctx, dirty) {
				ctx = new_ctx;
				const dnpentry_changes = {};
				if (dirty & /*filteredEntries*/ 32) dnpentry_changes.entry = /*entry*/ ctx[19];
				if (dirty & /*selectedEntries, filteredEntries*/ 48) dnpentry_changes.selected = /*selectedEntries*/ ctx[4].has(/*entry*/ ctx[19].artist.id);
				dnpentry.$set(dnpentry_changes);
			},
			i: function intro(local) {
				if (current) return;
				transition_in(dnpentry.$$.fragment, local);
				current = true;
			},
			o: function outro(local) {
				transition_out(dnpentry.$$.fragment, local);
				current = false;
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(first);
				}

				destroy_component(dnpentry, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_each_block$6.name,
			type: "each",
			source: "(211:8) {#each filteredEntries as entry (entry.artist.id)}",
			ctx
		});

		return block;
	}

	// (172:8) {:else}
	function create_else_block$c(ctx) {
		let svg;
		let path;
		let t0;
		let h3;
		let t2;
		let p;

		const block = {
			c: function create() {
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				h3 = element("h3");
				h3.textContent = "No artists match your search";
				t2 = space();
				p = element("p");
				p.textContent = "Try adjusting your search terms or filters.";
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z");
				add_location(path, file$d, 184, 12, 7691);
				attr_dev(svg, "class", "mx-auto h-12 w-12 text-gray-400");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$d, 183, 10, 7579);
				attr_dev(h3, "class", "mt-2 text-sm font-medium text-gray-900");
				add_location(h3, file$d, 186, 10, 7839);
				attr_dev(p, "class", "mt-1 text-sm text-gray-500");
				add_location(p, file$d, 187, 10, 7934);
			},
			m: function mount(target, anchor) {
				insert_dev(target, svg, anchor);
				append_dev(svg, path);
				insert_dev(target, t0, anchor);
				insert_dev(target, h3, anchor);
				insert_dev(target, t2, anchor);
				insert_dev(target, p, anchor);
			},
			p: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(svg);
					detach_dev(t0);
					detach_dev(h3);
					detach_dev(t2);
					detach_dev(p);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block$c.name,
			type: "else",
			source: "(172:8) {:else}",
			ctx
		});

		return block;
	}

	// (155:8) {#if $dnpStore.entries.length === 0}
	function create_if_block_4$8(ctx) {
		let svg0;
		let path0;
		let t0;
		let h3;
		let t2;
		let p;
		let t4;
		let div;
		let button;
		let svg1;
		let path1;
		let t5;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				svg0 = svg_element("svg");
				path0 = svg_element("path");
				t0 = space();
				h3 = element("h3");
				h3.textContent = "No artists in your DNP list";
				t2 = space();
				p = element("p");
				p.textContent = "Get started by adding artists you want to avoid.";
				t4 = space();
				div = element("div");
				button = element("button");
				svg1 = svg_element("svg");
				path1 = svg_element("path");
				t5 = text("\n              Add your first artist");
				attr_dev(path0, "stroke-linecap", "round");
				attr_dev(path0, "stroke-linejoin", "round");
				attr_dev(path0, "stroke-width", "2");
				attr_dev(path0, "d", "M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3");
				add_location(path0, file$d, 167, 12, 6443);
				attr_dev(svg0, "class", "mx-auto h-12 w-12 text-gray-400");
				attr_dev(svg0, "fill", "none");
				attr_dev(svg0, "viewBox", "0 0 24 24");
				attr_dev(svg0, "stroke", "currentColor");
				add_location(svg0, file$d, 166, 10, 6331);
				attr_dev(h3, "class", "mt-2 text-sm font-medium text-gray-900");
				add_location(h3, file$d, 169, 10, 6695);
				attr_dev(p, "class", "mt-1 text-sm text-gray-500");
				add_location(p, file$d, 170, 10, 6789);
				attr_dev(path1, "stroke-linecap", "round");
				attr_dev(path1, "stroke-linejoin", "round");
				attr_dev(path1, "stroke-width", "2");
				attr_dev(path1, "d", "M12 6v6m0 0v6m0-6h6m-6 0H6");
				add_location(path1, file$d, 177, 16, 7353);
				attr_dev(svg1, "class", "-ml-1 mr-2 h-5 w-5");
				attr_dev(svg1, "fill", "none");
				attr_dev(svg1, "viewBox", "0 0 24 24");
				attr_dev(svg1, "stroke", "currentColor");
				add_location(svg1, file$d, 176, 14, 7250);
				attr_dev(button, "class", "inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500");
				add_location(button, file$d, 172, 12, 6921);
				attr_dev(div, "class", "mt-6");
				add_location(div, file$d, 171, 10, 6890);
			},
			m: function mount(target, anchor) {
				insert_dev(target, svg0, anchor);
				append_dev(svg0, path0);
				insert_dev(target, t0, anchor);
				insert_dev(target, h3, anchor);
				insert_dev(target, t2, anchor);
				insert_dev(target, p, anchor);
				insert_dev(target, t4, anchor);
				insert_dev(target, div, anchor);
				append_dev(div, button);
				append_dev(button, svg1);
				append_dev(svg1, path1);
				append_dev(button, t5);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*click_handler_2*/ ctx[16], false, false, false, false);
					mounted = true;
				}
			},
			p: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(svg0);
					detach_dev(t0);
					detach_dev(h3);
					detach_dev(t2);
					detach_dev(p);
					detach_dev(t4);
					detach_dev(div);
				}

				mounted = false;
				dispose();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_4$8.name,
			type: "if",
			source: "(155:8) {#if $dnpStore.entries.length === 0}",
			ctx
		});

		return block;
	}

	// (223:2) {#if $dnpStore.entries.length > 0}
	function create_if_block$d(ctx) {
		let div10;
		let div9;
		let div2;
		let div0;
		let t0_value = /*$dnpStore*/ ctx[2].entries.length + "";
		let t0;
		let t1;
		let div1;
		let t3;
		let div5;
		let div3;
		let t4_value = /*$dnpTags*/ ctx[6].length + "";
		let t4;
		let t5;
		let div4;
		let t7;
		let div8;
		let div6;
		let t8_value = /*$dnpStore*/ ctx[2].entries.filter(func).length + "";
		let t8;
		let t9;
		let div7;

		const block = {
			c: function create() {
				div10 = element("div");
				div9 = element("div");
				div2 = element("div");
				div0 = element("div");
				t0 = text(t0_value);
				t1 = space();
				div1 = element("div");
				div1.textContent = "Total Artists";
				t3 = space();
				div5 = element("div");
				div3 = element("div");
				t4 = text(t4_value);
				t5 = space();
				div4 = element("div");
				div4.textContent = "Unique Tags";
				t7 = space();
				div8 = element("div");
				div6 = element("div");
				t8 = text(t8_value);
				t9 = space();
				div7 = element("div");
				div7.textContent = "With Notes";
				attr_dev(div0, "class", "text-2xl font-bold text-gray-900");
				add_location(div0, file$d, 237, 10, 9742);
				attr_dev(div1, "class", "text-sm text-gray-500");
				add_location(div1, file$d, 238, 10, 9831);
				attr_dev(div2, "class", "text-center");
				add_location(div2, file$d, 236, 8, 9706);
				attr_dev(div3, "class", "text-2xl font-bold text-gray-900");
				add_location(div3, file$d, 241, 10, 9945);
				attr_dev(div4, "class", "text-sm text-gray-500");
				add_location(div4, file$d, 242, 10, 10025);
				attr_dev(div5, "class", "text-center");
				add_location(div5, file$d, 240, 8, 9909);
				attr_dev(div6, "class", "text-2xl font-bold text-gray-900");
				add_location(div6, file$d, 245, 10, 10137);
				attr_dev(div7, "class", "text-sm text-gray-500");
				add_location(div7, file$d, 248, 10, 10270);
				attr_dev(div8, "class", "text-center");
				add_location(div8, file$d, 244, 8, 10101);
				attr_dev(div9, "class", "grid grid-cols-1 gap-4 sm:grid-cols-3");
				add_location(div9, file$d, 235, 6, 9646);
				attr_dev(div10, "class", "mt-6 bg-gray-50 rounded-lg p-4");
				add_location(div10, file$d, 234, 4, 9595);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div10, anchor);
				append_dev(div10, div9);
				append_dev(div9, div2);
				append_dev(div2, div0);
				append_dev(div0, t0);
				append_dev(div2, t1);
				append_dev(div2, div1);
				append_dev(div9, t3);
				append_dev(div9, div5);
				append_dev(div5, div3);
				append_dev(div3, t4);
				append_dev(div5, t5);
				append_dev(div5, div4);
				append_dev(div9, t7);
				append_dev(div9, div8);
				append_dev(div8, div6);
				append_dev(div6, t8);
				append_dev(div8, t9);
				append_dev(div8, div7);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*$dnpStore*/ 4 && t0_value !== (t0_value = /*$dnpStore*/ ctx[2].entries.length + "")) set_data_dev(t0, t0_value);
				if (dirty & /*$dnpTags*/ 64 && t4_value !== (t4_value = /*$dnpTags*/ ctx[6].length + "")) set_data_dev(t4, t4_value);
				if (dirty & /*$dnpStore*/ 4 && t8_value !== (t8_value = /*$dnpStore*/ ctx[2].entries.filter(func).length + "")) set_data_dev(t8, t8_value);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div10);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block$d.name,
			type: "if",
			source: "(223:2) {#if $dnpStore.entries.length > 0}",
			ctx
		});

		return block;
	}

	function create_fragment$d(ctx) {
		let div10;
		let div2;
		let div1;
		let div0;
		let h2;
		let t1;
		let p;
		let t3;
		let button;
		let svg0;
		let path0;
		let t4;
		let t5;
		let t6;
		let div8;
		let div7;
		let div5;
		let label0;
		let t8;
		let div4;
		let div3;
		let svg1;
		let path1;
		let t9;
		let input;
		let t10;
		let div6;
		let label1;
		let t12;
		let select;
		let option;
		let t14;
		let t15;
		let div9;
		let current_block_type_index;
		let if_block2;
		let t16;
		let current;
		let mounted;
		let dispose;
		let if_block0 = /*showAddForm*/ ctx[3] && create_if_block_8$4(ctx);
		let each_value_1 = ensure_array_like_dev(/*$dnpTags*/ ctx[6]);
		let each_blocks = [];

		for (let i = 0; i < each_value_1.length; i += 1) {
			each_blocks[i] = create_each_block_1$2(get_each_context_1$2(ctx, each_value_1, i));
		}

		let if_block1 = /*selectedEntries*/ ctx[4].size > 0 && create_if_block_7$6(ctx);
		const if_block_creators = [create_if_block_1$c, create_if_block_2$9, create_if_block_3$8, create_else_block_1$6];
		const if_blocks = [];

		function select_block_type(ctx, dirty) {
			if (/*$dnpStore*/ ctx[2].isLoading) return 0;
			if (/*$dnpStore*/ ctx[2].error) return 1;
			if (/*filteredEntries*/ ctx[5].length === 0) return 2;
			return 3;
		}

		current_block_type_index = select_block_type(ctx);
		if_block2 = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);
		let if_block3 = /*$dnpStore*/ ctx[2].entries.length > 0 && create_if_block$d(ctx);

		const block = {
			c: function create() {
				div10 = element("div");
				div2 = element("div");
				div1 = element("div");
				div0 = element("div");
				h2 = element("h2");
				h2.textContent = "Do-Not-Play List";
				t1 = space();
				p = element("p");
				p.textContent = "Manage artists you want to avoid across your streaming services.";
				t3 = space();
				button = element("button");
				svg0 = svg_element("svg");
				path0 = svg_element("path");
				t4 = text("\n        Add Artist");
				t5 = space();
				if (if_block0) if_block0.c();
				t6 = space();
				div8 = element("div");
				div7 = element("div");
				div5 = element("div");
				label0 = element("label");
				label0.textContent = "Search artists";
				t8 = space();
				div4 = element("div");
				div3 = element("div");
				svg1 = svg_element("svg");
				path1 = svg_element("path");
				t9 = space();
				input = element("input");
				t10 = space();
				div6 = element("div");
				label1 = element("label");
				label1.textContent = "Filter by tag";
				t12 = space();
				select = element("select");
				option = element("option");
				option.textContent = "All tags";

				for (let i = 0; i < each_blocks.length; i += 1) {
					each_blocks[i].c();
				}

				t14 = space();
				if (if_block1) if_block1.c();
				t15 = space();
				div9 = element("div");
				if_block2.c();
				t16 = space();
				if (if_block3) if_block3.c();
				attr_dev(h2, "class", "text-2xl font-bold text-gray-900");
				add_location(h2, file$d, 67, 8, 2027);
				attr_dev(p, "class", "mt-1 text-sm text-gray-600");
				add_location(p, file$d, 68, 8, 2102);
				add_location(div0, file$d, 66, 6, 2013);
				attr_dev(path0, "stroke-linecap", "round");
				attr_dev(path0, "stroke-linejoin", "round");
				attr_dev(path0, "stroke-width", "2");
				attr_dev(path0, "d", "M12 6v6m0 0v6m0-6h6m-6 0H6");
				add_location(path0, file$d, 77, 10, 2658);
				attr_dev(svg0, "class", "-ml-1 mr-2 h-5 w-5");
				attr_dev(svg0, "fill", "none");
				attr_dev(svg0, "viewBox", "0 0 24 24");
				attr_dev(svg0, "stroke", "currentColor");
				add_location(svg0, file$d, 76, 8, 2561);
				attr_dev(button, "class", "inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500");
				add_location(button, file$d, 72, 6, 2248);
				attr_dev(div1, "class", "flex justify-between items-center");
				add_location(div1, file$d, 65, 4, 1959);
				attr_dev(div2, "class", "mb-6");
				add_location(div2, file$d, 64, 2, 1936);
				attr_dev(label0, "for", "search");
				attr_dev(label0, "class", "sr-only");
				add_location(label0, file$d, 96, 8, 3264);
				attr_dev(path1, "stroke-linecap", "round");
				attr_dev(path1, "stroke-linejoin", "round");
				attr_dev(path1, "stroke-width", "2");
				attr_dev(path1, "d", "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z");
				add_location(path1, file$d, 100, 14, 3563);
				attr_dev(svg1, "class", "h-5 w-5 text-gray-400");
				attr_dev(svg1, "fill", "none");
				attr_dev(svg1, "viewBox", "0 0 24 24");
				attr_dev(svg1, "stroke", "currentColor");
				add_location(svg1, file$d, 99, 12, 3459);
				attr_dev(div3, "class", "absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none");
				add_location(div3, file$d, 98, 10, 3364);
				attr_dev(input, "id", "search");
				attr_dev(input, "type", "text");
				attr_dev(input, "placeholder", "Search artists, tags, or notes...");
				attr_dev(input, "class", "block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md leading-5 bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm");
				add_location(input, file$d, 103, 10, 3730);
				attr_dev(div4, "class", "relative");
				add_location(div4, file$d, 97, 8, 3331);
				attr_dev(div5, "class", "flex-1");
				add_location(div5, file$d, 95, 6, 3235);
				attr_dev(label1, "for", "tag-filter");
				attr_dev(label1, "class", "sr-only");
				add_location(label1, file$d, 114, 8, 4205);
				option.__value = "";
				set_input_value(option, option.__value);
				add_location(option, file$d, 120, 10, 4531);
				attr_dev(select, "id", "tag-filter");
				attr_dev(select, "class", "block w-full pl-3 pr-10 py-2 text-base border border-gray-300 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm rounded-md");
				if (/*selectedTag*/ ctx[1] === void 0) add_render_callback(() => /*select_change_handler*/ ctx[14].call(select));
				add_location(select, file$d, 115, 8, 4275);
				attr_dev(div6, "class", "sm:w-48");
				add_location(div6, file$d, 113, 6, 4175);
				attr_dev(div7, "class", "flex flex-col sm:flex-row gap-4");
				add_location(div7, file$d, 94, 4, 3183);
				attr_dev(div8, "class", "mb-6 bg-white shadow rounded-lg p-4");
				add_location(div8, file$d, 93, 2, 3129);
				attr_dev(div9, "class", "bg-white shadow overflow-hidden sm:rounded-md");
				add_location(div9, file$d, 141, 2, 4997);
				attr_dev(div10, "class", "px-4 py-6 sm:px-0");
				add_location(div10, file$d, 63, 0, 1902);
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				insert_dev(target, div10, anchor);
				append_dev(div10, div2);
				append_dev(div2, div1);
				append_dev(div1, div0);
				append_dev(div0, h2);
				append_dev(div0, t1);
				append_dev(div0, p);
				append_dev(div1, t3);
				append_dev(div1, button);
				append_dev(button, svg0);
				append_dev(svg0, path0);
				append_dev(button, t4);
				append_dev(div10, t5);
				if (if_block0) if_block0.m(div10, null);
				append_dev(div10, t6);
				append_dev(div10, div8);
				append_dev(div8, div7);
				append_dev(div7, div5);
				append_dev(div5, label0);
				append_dev(div5, t8);
				append_dev(div5, div4);
				append_dev(div4, div3);
				append_dev(div3, svg1);
				append_dev(svg1, path1);
				append_dev(div4, t9);
				append_dev(div4, input);
				set_input_value(input, /*searchQuery*/ ctx[0]);
				append_dev(div7, t10);
				append_dev(div7, div6);
				append_dev(div6, label1);
				append_dev(div6, t12);
				append_dev(div6, select);
				append_dev(select, option);

				for (let i = 0; i < each_blocks.length; i += 1) {
					if (each_blocks[i]) {
						each_blocks[i].m(select, null);
					}
				}

				select_option(select, /*selectedTag*/ ctx[1], true);
				append_dev(div10, t14);
				if (if_block1) if_block1.m(div10, null);
				append_dev(div10, t15);
				append_dev(div10, div9);
				if_blocks[current_block_type_index].m(div9, null);
				append_dev(div10, t16);
				if (if_block3) if_block3.m(div10, null);
				current = true;

				if (!mounted) {
					dispose = [
						listen_dev(button, "click", /*click_handler*/ ctx[12], false, false, false, false),
						listen_dev(input, "input", /*input_input_handler*/ ctx[13]),
						listen_dev(select, "change", /*select_change_handler*/ ctx[14])
					];

					mounted = true;
				}
			},
			p: function update(ctx, [dirty]) {
				if (/*showAddForm*/ ctx[3]) {
					if (if_block0) {
						if_block0.p(ctx, dirty);

						if (dirty & /*showAddForm*/ 8) {
							transition_in(if_block0, 1);
						}
					} else {
						if_block0 = create_if_block_8$4(ctx);
						if_block0.c();
						transition_in(if_block0, 1);
						if_block0.m(div10, t6);
					}
				} else if (if_block0) {
					group_outros();

					transition_out(if_block0, 1, 1, () => {
						if_block0 = null;
					});

					check_outros();
				}

				if (dirty & /*searchQuery*/ 1 && input.value !== /*searchQuery*/ ctx[0]) {
					set_input_value(input, /*searchQuery*/ ctx[0]);
				}

				if (dirty & /*$dnpTags*/ 64) {
					each_value_1 = ensure_array_like_dev(/*$dnpTags*/ ctx[6]);
					let i;

					for (i = 0; i < each_value_1.length; i += 1) {
						const child_ctx = get_each_context_1$2(ctx, each_value_1, i);

						if (each_blocks[i]) {
							each_blocks[i].p(child_ctx, dirty);
						} else {
							each_blocks[i] = create_each_block_1$2(child_ctx);
							each_blocks[i].c();
							each_blocks[i].m(select, null);
						}
					}

					for (; i < each_blocks.length; i += 1) {
						each_blocks[i].d(1);
					}

					each_blocks.length = each_value_1.length;
				}

				if (dirty & /*selectedTag, $dnpTags*/ 66) {
					select_option(select, /*selectedTag*/ ctx[1]);
				}

				if (/*selectedEntries*/ ctx[4].size > 0) {
					if (if_block1) {
						if_block1.p(ctx, dirty);

						if (dirty & /*selectedEntries*/ 16) {
							transition_in(if_block1, 1);
						}
					} else {
						if_block1 = create_if_block_7$6(ctx);
						if_block1.c();
						transition_in(if_block1, 1);
						if_block1.m(div10, t15);
					}
				} else if (if_block1) {
					group_outros();

					transition_out(if_block1, 1, 1, () => {
						if_block1 = null;
					});

					check_outros();
				}

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
					if_block2 = if_blocks[current_block_type_index];

					if (!if_block2) {
						if_block2 = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);
						if_block2.c();
					} else {
						if_block2.p(ctx, dirty);
					}

					transition_in(if_block2, 1);
					if_block2.m(div9, null);
				}

				if (/*$dnpStore*/ ctx[2].entries.length > 0) {
					if (if_block3) {
						if_block3.p(ctx, dirty);
					} else {
						if_block3 = create_if_block$d(ctx);
						if_block3.c();
						if_block3.m(div10, null);
					}
				} else if (if_block3) {
					if_block3.d(1);
					if_block3 = null;
				}
			},
			i: function intro(local) {
				if (current) return;
				transition_in(if_block0);
				transition_in(if_block1);
				transition_in(if_block2);
				current = true;
			},
			o: function outro(local) {
				transition_out(if_block0);
				transition_out(if_block1);
				transition_out(if_block2);
				current = false;
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div10);
				}

				if (if_block0) if_block0.d();
				destroy_each(each_blocks, detaching);
				if (if_block1) if_block1.d();
				if_blocks[current_block_type_index].d();
				if (if_block3) if_block3.d();
				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_fragment$d.name,
			type: "component",
			source: "",
			ctx
		});

		return block;
	}

	const func = e => e.note;

	function instance$d($$self, $$props, $$invalidate) {
		let filteredEntries;
		let $dnpStore;
		let $dnpTags;
		validate_store(dnpStore, 'dnpStore');
		component_subscribe($$self, dnpStore, $$value => $$invalidate(2, $dnpStore = $$value));
		validate_store(dnpTags, 'dnpTags');
		component_subscribe($$self, dnpTags, $$value => $$invalidate(6, $dnpTags = $$value));
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('DnpManager', slots, []);
		let searchQuery = '';
		let selectedTag = '';
		let showAddForm = false;
		let selectedEntries = new Set();

		function toggleSelectAll() {
			if (selectedEntries.size === filteredEntries.length) {
				selectedEntries.clear();
			} else {
				$$invalidate(4, selectedEntries = new Set(filteredEntries.map(entry => entry.artist.id)));
			}

			$$invalidate(4, selectedEntries); // Trigger reactivity
		}

		function toggleSelectEntry(artistId) {
			if (selectedEntries.has(artistId)) {
				selectedEntries.delete(artistId);
			} else {
				selectedEntries.add(artistId);
			}

			$$invalidate(4, selectedEntries); // Trigger reactivity
		}

		function clearSelection() {
			selectedEntries.clear();
			$$invalidate(4, selectedEntries); // Trigger reactivity
		}

		async function handleBulkDelete() {
			if (selectedEntries.size === 0) return;

			if (confirm(`Are you sure you want to remove ${selectedEntries.size} artist(s) from your DNP list?`)) {
				const promises = Array.from(selectedEntries).map(artistId => dnpActions.removeArtist(artistId));
				await Promise.all(promises);
				clearSelection();
			}
		}

		function handleArtistAdded() {
			$$invalidate(3, showAddForm = false);
		}

		const writable_props = [];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<DnpManager> was created with unknown prop '${key}'`);
		});

		const click_handler = () => $$invalidate(3, showAddForm = !showAddForm);

		function input_input_handler() {
			searchQuery = this.value;
			$$invalidate(0, searchQuery);
		}

		function select_change_handler() {
			selectedTag = select_value(this);
			$$invalidate(1, selectedTag);
		}

		const click_handler_1 = () => dnpActions.fetchDnpList();
		const click_handler_2 = () => $$invalidate(3, showAddForm = true);

		const click_handler_3 = () => {
			$$invalidate(0, searchQuery = '');
			$$invalidate(1, selectedTag = '');
		};

		const toggleSelect_handler = entry => toggleSelectEntry(entry.artist.id);

		$$self.$capture_state = () => ({
			dnpActions,
			dnpStore,
			dnpTags,
			ArtistSearch,
			DnpEntry,
			BulkActions,
			searchQuery,
			selectedTag,
			showAddForm,
			selectedEntries,
			toggleSelectAll,
			toggleSelectEntry,
			clearSelection,
			handleBulkDelete,
			handleArtistAdded,
			filteredEntries,
			$dnpStore,
			$dnpTags
		});

		$$self.$inject_state = $$props => {
			if ('searchQuery' in $$props) $$invalidate(0, searchQuery = $$props.searchQuery);
			if ('selectedTag' in $$props) $$invalidate(1, selectedTag = $$props.selectedTag);
			if ('showAddForm' in $$props) $$invalidate(3, showAddForm = $$props.showAddForm);
			if ('selectedEntries' in $$props) $$invalidate(4, selectedEntries = $$props.selectedEntries);
			if ('filteredEntries' in $$props) $$invalidate(5, filteredEntries = $$props.filteredEntries);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		$$self.$$.update = () => {
			if ($$self.$$.dirty & /*$dnpStore, searchQuery, selectedTag*/ 7) {
				$$invalidate(5, filteredEntries = $dnpStore.entries.filter(entry => {
					const matchesSearch = !searchQuery || entry.artist.canonical_name.toLowerCase().includes(searchQuery.toLowerCase()) || entry.tags.some(tag => tag.toLowerCase().includes(searchQuery.toLowerCase())) || entry.note && entry.note.toLowerCase().includes(searchQuery.toLowerCase());
					const matchesTag = !selectedTag || entry.tags.includes(selectedTag);
					return matchesSearch && matchesTag;
				}));
			}
		};

		return [
			searchQuery,
			selectedTag,
			$dnpStore,
			showAddForm,
			selectedEntries,
			filteredEntries,
			$dnpTags,
			toggleSelectAll,
			toggleSelectEntry,
			clearSelection,
			handleBulkDelete,
			handleArtistAdded,
			click_handler,
			input_input_handler,
			select_change_handler,
			click_handler_1,
			click_handler_2,
			click_handler_3,
			toggleSelect_handler
		];
	}

	class DnpManager extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance$d, create_fragment$d, safe_not_equal, {});

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "DnpManager",
				options,
				id: create_fragment$d.name
			});
		}
	}

	const defaultOptions = {
	    aggressiveness: 'moderate',
	    blockCollabs: true,
	    blockFeaturing: true,
	    blockSongwriterOnly: false,
	};
	const initialState$1 = {
	    currentPlan: null,
	    isPlanning: false,
	    isExecuting: false,
	    currentBatch: null,
	    actionHistory: [],
	    options: defaultOptions,
	    error: null,
	};
	const enforcementStore = writable(initialState$1);
	const hasActivePlan = derived(enforcementStore, ($enforcement) => $enforcement.currentPlan !== null);
	const executionProgress = derived(enforcementStore, ($enforcement) => {
	    if (!$enforcement.currentBatch)
	        return null;
	    const { totalItems, completedItems, failedItems, skippedItems } = $enforcement.currentBatch.summary;
	    const processedItems = completedItems + failedItems + skippedItems;
	    return {
	        total: totalItems,
	        processed: processedItems,
	        completed: completedItems,
	        failed: failedItems,
	        skipped: skippedItems,
	        percentage: totalItems > 0 ? Math.round((processedItems / totalItems) * 100) : 0,
	    };
	});
	const canRollback = derived(enforcementStore, ($enforcement) => $enforcement.actionHistory.some(batch => batch.status === 'completed' && batch.items.some(item => item.status === 'completed')));
	// Enforcement actions
	const enforcementActions = {
	    updateOptions: (options) => {
	        enforcementStore.update(state => ({
	            ...state,
	            options: { ...state.options, ...options },
	        }));
	    },
	    createPlan: async (providers, dryRun = true) => {
	        let currentOptions = defaultOptions;
	        enforcementStore.update(state => {
	            currentOptions = state.options;
	            return { ...state, isPlanning: true, error: null };
	        });
	        try {
	            const token = localStorage.getItem('auth_token');
	            const response = await fetch('http://localhost:3000/api/v1/spotify/library/plan', {
	                method: 'POST',
	                headers: {
	                    'Authorization': `Bearer ${token}`,
	                    'Content-Type': 'application/json',
	                },
	                body: JSON.stringify({
	                    providers,
	                    options: currentOptions,
	                    dryRun,
	                }),
	            });
	            const result = await response.json();
	            if (result.success) {
	                enforcementStore.update(state => ({
	                    ...state,
	                    currentPlan: result.data,
	                    isPlanning: false,
	                }));
	                return { success: true, data: result.data };
	            }
	            else {
	                enforcementStore.update(state => ({
	                    ...state,
	                    error: result.message,
	                    isPlanning: false,
	                }));
	                return { success: false, message: result.message };
	            }
	        }
	        catch (error) {
	            enforcementStore.update(state => ({
	                ...state,
	                error: 'Failed to create enforcement plan',
	                isPlanning: false,
	            }));
	            return { success: false, message: 'Failed to create enforcement plan' };
	        }
	    },
	    executePlan: async (planId) => {
	        enforcementStore.update(state => ({ ...state, isExecuting: true, error: null }));
	        try {
	            const token = localStorage.getItem('auth_token');
	            const response = await fetch('http://localhost:3000/api/v1/spotify/enforcement/execute', {
	                method: 'POST',
	                headers: {
	                    'Authorization': `Bearer ${token}`,
	                    'Content-Type': 'application/json',
	                },
	                body: JSON.stringify({
	                    planId,
	                    dryRun: false,
	                }),
	            });
	            const result = await response.json();
	            if (result.success) {
	                const batch = result.data;
	                enforcementStore.update(state => ({
	                    ...state,
	                    currentBatch: batch,
	                    isExecuting: false,
	                }));
	                // Start polling for progress
	                enforcementActions.pollProgress(batch.id);
	                return { success: true, data: batch };
	            }
	            else {
	                enforcementStore.update(state => ({
	                    ...state,
	                    error: result.message,
	                    isExecuting: false,
	                }));
	                return { success: false, message: result.message };
	            }
	        }
	        catch (error) {
	            enforcementStore.update(state => ({
	                ...state,
	                error: 'Failed to execute enforcement plan',
	                isExecuting: false,
	            }));
	            return { success: false, message: 'Failed to execute enforcement plan' };
	        }
	    },
	    pollProgress: async (batchId) => {
	        const pollInterval = setInterval(async () => {
	            try {
	                const token = localStorage.getItem('auth_token');
	                const response = await fetch(`http://localhost:3000/api/v1/spotify/enforcement/progress/${batchId}`, {
	                    headers: {
	                        'Authorization': `Bearer ${token}`,
	                    },
	                });
	                const result = await response.json();
	                if (result.success) {
	                    const batch = result.data;
	                    enforcementStore.update(state => ({
	                        ...state,
	                        currentBatch: batch,
	                    }));
	                    // Stop polling if batch is complete
	                    if (batch.status === 'completed' || batch.status === 'failed' || batch.status === 'cancelled') {
	                        clearInterval(pollInterval);
	                        // Move to history
	                        enforcementStore.update(state => ({
	                            ...state,
	                            actionHistory: [batch, ...state.actionHistory],
	                            currentBatch: null,
	                            currentPlan: null,
	                        }));
	                    }
	                }
	            }
	            catch (error) {
	                console.error('Failed to poll progress:', error);
	            }
	        }, 2000); // Poll every 2 seconds
	    },
	    rollbackBatch: async (batchId) => {
	        try {
	            const token = localStorage.getItem('auth_token');
	            const response = await fetch('http://localhost:3000/api/v1/spotify/enforcement/rollback', {
	                method: 'POST',
	                headers: {
	                    'Authorization': `Bearer ${token}`,
	                    'Content-Type': 'application/json',
	                },
	                body: JSON.stringify({ batchId }),
	            });
	            const result = await response.json();
	            if (result.success) {
	                // Refresh action history
	                await enforcementActions.fetchActionHistory();
	                return { success: true };
	            }
	            else {
	                return { success: false, message: result.message };
	            }
	        }
	        catch (error) {
	            return { success: false, message: 'Failed to rollback actions' };
	        }
	    },
	    fetchActionHistory: async () => {
	        try {
	            const token = localStorage.getItem('auth_token');
	            const response = await fetch('http://localhost:3000/api/v1/spotify/enforcement/history', {
	                headers: {
	                    'Authorization': `Bearer ${token}`,
	                },
	            });
	            const result = await response.json();
	            if (result.success) {
	                enforcementStore.update(state => ({
	                    ...state,
	                    actionHistory: result.data,
	                }));
	            }
	        }
	        catch (error) {
	            console.error('Failed to fetch action history:', error);
	        }
	    },
	    clearPlan: () => {
	        enforcementStore.update(state => ({
	            ...state,
	            currentPlan: null,
	            error: null,
	        }));
	    },
	    clearError: () => {
	        enforcementStore.update(state => ({
	            ...state,
	            error: null,
	        }));
	    },
	};

	/* src/lib/components/EnforcementOptions.svelte generated by Svelte v4.2.20 */
	const file$c = "src/lib/components/EnforcementOptions.svelte";

	// (145:2) {#if options.aggressiveness === 'aggressive' || options.blockSongwriterOnly}
	function create_if_block$c(ctx) {
		let div4;
		let div3;
		let div0;
		let svg;
		let path;
		let t0;
		let div2;
		let h3;
		let t2;
		let div1;
		let p;

		const block = {
			c: function create() {
				div4 = element("div");
				div3 = element("div");
				div0 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				div2 = element("div");
				h3 = element("h3");
				h3.textContent = "Aggressive Settings Warning";
				t2 = space();
				div1 = element("div");
				p = element("p");
				p.textContent = "These settings may remove a significant amount of content from your library. \n              We recommend reviewing the enforcement preview carefully before executing.";
				attr_dev(path, "fill-rule", "evenodd");
				attr_dev(path, "d", "M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z");
				attr_dev(path, "clip-rule", "evenodd");
				add_location(path, file$c, 153, 12, 5699);
				attr_dev(svg, "class", "h-5 w-5 text-yellow-400");
				attr_dev(svg, "viewBox", "0 0 20 20");
				attr_dev(svg, "fill", "currentColor");
				add_location(svg, file$c, 152, 10, 5609);
				attr_dev(div0, "class", "flex-shrink-0");
				add_location(div0, file$c, 151, 8, 5571);
				attr_dev(h3, "class", "text-sm font-medium text-yellow-800");
				add_location(h3, file$c, 157, 10, 6031);
				add_location(p, file$c, 161, 12, 6201);
				attr_dev(div1, "class", "mt-2 text-sm text-yellow-700");
				add_location(div1, file$c, 160, 10, 6146);
				attr_dev(div2, "class", "ml-3");
				add_location(div2, file$c, 156, 8, 6002);
				attr_dev(div3, "class", "flex");
				add_location(div3, file$c, 150, 6, 5544);
				attr_dev(div4, "class", "bg-yellow-50 border border-yellow-200 rounded-md p-4");
				add_location(div4, file$c, 149, 4, 5471);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div4, anchor);
				append_dev(div4, div3);
				append_dev(div3, div0);
				append_dev(div0, svg);
				append_dev(svg, path);
				append_dev(div3, t0);
				append_dev(div3, div2);
				append_dev(div2, h3);
				append_dev(div2, t2);
				append_dev(div2, div1);
				append_dev(div1, p);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div4);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block$c.name,
			type: "if",
			source: "(145:2) {#if options.aggressiveness === 'aggressive' || options.blockSongwriterOnly}",
			ctx
		});

		return block;
	}

	function create_fragment$c(ctx) {
		let div19;
		let div7;
		let h40;
		let t1;
		let p0;
		let t3;
		let fieldset;
		let legend;
		let t5;
		let div6;
		let div0;
		let input0;
		let input0_checked_value;
		let t6;
		let label0;
		let t8;
		let div1;
		let t10;
		let div2;
		let input1;
		let input1_checked_value;
		let t11;
		let label1;
		let t13;
		let div3;
		let t15;
		let div4;
		let input2;
		let input2_checked_value;
		let t16;
		let label2;
		let t18;
		let div5;
		let t20;
		let div18;
		let h41;
		let t22;
		let p1;
		let t24;
		let div17;
		let div10;
		let div8;
		let input3;
		let input3_checked_value;
		let t25;
		let div9;
		let label3;
		let t27;
		let p2;
		let t29;
		let div13;
		let div11;
		let input4;
		let input4_checked_value;
		let t30;
		let div12;
		let label4;
		let t32;
		let p3;
		let t34;
		let div16;
		let div14;
		let input5;
		let input5_checked_value;
		let t35;
		let div15;
		let label5;
		let t37;
		let p4;
		let t39;
		let mounted;
		let dispose;
		let if_block = (/*options*/ ctx[0].aggressiveness === 'aggressive' || /*options*/ ctx[0].blockSongwriterOnly) && create_if_block$c(ctx);

		const block = {
			c: function create() {
				div19 = element("div");
				div7 = element("div");
				h40 = element("h4");
				h40.textContent = "Enforcement Aggressiveness";
				t1 = space();
				p0 = element("p");
				p0.textContent = "Choose how thoroughly to apply your blocklist across your music library.";
				t3 = space();
				fieldset = element("fieldset");
				legend = element("legend");
				legend.textContent = "Aggressiveness level";
				t5 = space();
				div6 = element("div");
				div0 = element("div");
				input0 = element("input");
				t6 = space();
				label0 = element("label");
				label0.textContent = "Conservative";
				t8 = space();
				div1 = element("div");
				div1.textContent = "Only remove explicitly saved/liked content. Preserves playlists and recommendations.";
				t10 = space();
				div2 = element("div");
				input1 = element("input");
				t11 = space();
				label1 = element("label");
				label1.textContent = "Moderate (Recommended)";
				t13 = space();
				div3 = element("div");
				div3.textContent = "Remove from saved content and playlists. Filters recommendations where possible.";
				t15 = space();
				div4 = element("div");
				input2 = element("input");
				t16 = space();
				label2 = element("label");
				label2.textContent = "Aggressive";
				t18 = space();
				div5 = element("div");
				div5.textContent = "Maximum removal including radio seeds, recommendations, and related content.";
				t20 = space();
				div18 = element("div");
				h41 = element("h4");
				h41.textContent = "Collaboration Handling";
				t22 = space();
				p1 = element("p");
				p1.textContent = "Configure how to handle songs where blocked artists appear as collaborators or featured artists.";
				t24 = space();
				div17 = element("div");
				div10 = element("div");
				div8 = element("div");
				input3 = element("input");
				t25 = space();
				div9 = element("div");
				label3 = element("label");
				label3.textContent = "Block collaborations";
				t27 = space();
				p2 = element("p");
				p2.textContent = "Remove songs where blocked artists are listed as collaborators or co-writers.";
				t29 = space();
				div13 = element("div");
				div11 = element("div");
				input4 = element("input");
				t30 = space();
				div12 = element("div");
				label4 = element("label");
				label4.textContent = "Block featuring";
				t32 = space();
				p3 = element("p");
				p3.textContent = "Remove songs where blocked artists are featured (e.g., \"Song Title (feat. Blocked Artist)\").";
				t34 = space();
				div16 = element("div");
				div14 = element("div");
				input5 = element("input");
				t35 = space();
				div15 = element("div");
				label5 = element("label");
				label5.textContent = "Block songwriter credits only";
				t37 = space();
				p4 = element("p");
				p4.textContent = "Remove songs where blocked artists are credited only as songwriters (most restrictive).";
				t39 = space();
				if (if_block) if_block.c();
				attr_dev(h40, "class", "text-base font-medium text-gray-900");
				add_location(h40, file$c, 17, 4, 425);
				attr_dev(p0, "class", "text-sm leading-5 text-gray-500");
				add_location(p0, file$c, 18, 4, 509);
				attr_dev(legend, "class", "sr-only");
				add_location(legend, file$c, 22, 6, 675);
				attr_dev(input0, "id", "conservative");
				attr_dev(input0, "name", "aggressiveness");
				attr_dev(input0, "type", "radio");
				input0.checked = input0_checked_value = /*options*/ ctx[0].aggressiveness === 'conservative';
				attr_dev(input0, "class", "focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300");
				add_location(input0, file$c, 25, 10, 809);
				attr_dev(label0, "for", "conservative");
				attr_dev(label0, "class", "ml-3 block text-sm font-medium text-gray-700");
				add_location(label0, file$c, 33, 10, 1141);
				attr_dev(div0, "class", "flex items-center");
				add_location(div0, file$c, 24, 8, 767);
				attr_dev(div1, "class", "ml-7 text-sm text-gray-500");
				add_location(div1, file$c, 37, 8, 1288);
				attr_dev(input1, "id", "moderate");
				attr_dev(input1, "name", "aggressiveness");
				attr_dev(input1, "type", "radio");
				input1.checked = input1_checked_value = /*options*/ ctx[0].aggressiveness === 'moderate';
				attr_dev(input1, "class", "focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300");
				add_location(input1, file$c, 42, 10, 1490);
				attr_dev(label1, "for", "moderate");
				attr_dev(label1, "class", "ml-3 block text-sm font-medium text-gray-700");
				add_location(label1, file$c, 50, 10, 1810);
				attr_dev(div2, "class", "flex items-center");
				add_location(div2, file$c, 41, 8, 1448);
				attr_dev(div3, "class", "ml-7 text-sm text-gray-500");
				add_location(div3, file$c, 54, 8, 1963);
				attr_dev(input2, "id", "aggressive");
				attr_dev(input2, "name", "aggressiveness");
				attr_dev(input2, "type", "radio");
				input2.checked = input2_checked_value = /*options*/ ctx[0].aggressiveness === 'aggressive';
				attr_dev(input2, "class", "focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300");
				add_location(input2, file$c, 59, 10, 2161);
				attr_dev(label2, "for", "aggressive");
				attr_dev(label2, "class", "ml-3 block text-sm font-medium text-gray-700");
				add_location(label2, file$c, 67, 10, 2487);
				attr_dev(div4, "class", "flex items-center");
				add_location(div4, file$c, 58, 8, 2119);
				attr_dev(div5, "class", "ml-7 text-sm text-gray-500");
				add_location(div5, file$c, 71, 8, 2630);
				attr_dev(div6, "class", "space-y-4");
				add_location(div6, file$c, 23, 6, 735);
				attr_dev(fieldset, "class", "mt-4");
				add_location(fieldset, file$c, 21, 4, 645);
				add_location(div7, file$c, 16, 2, 415);
				attr_dev(h41, "class", "text-base font-medium text-gray-900");
				add_location(h41, file$c, 80, 4, 2871);
				attr_dev(p1, "class", "text-sm leading-5 text-gray-500");
				add_location(p1, file$c, 81, 4, 2951);
				attr_dev(input3, "id", "block-collabs");
				attr_dev(input3, "type", "checkbox");
				input3.checked = input3_checked_value = /*options*/ ctx[0].blockCollabs;
				attr_dev(input3, "class", "focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300 rounded");
				add_location(input3, file$c, 87, 10, 3231);
				attr_dev(div8, "class", "flex items-center h-5");
				add_location(div8, file$c, 86, 8, 3185);
				attr_dev(label3, "for", "block-collabs");
				attr_dev(label3, "class", "font-medium text-gray-700");
				add_location(label3, file$c, 96, 10, 3562);
				attr_dev(p2, "class", "text-gray-500");
				add_location(p2, file$c, 99, 10, 3686);
				attr_dev(div9, "class", "ml-3 text-sm");
				add_location(div9, file$c, 95, 8, 3525);
				attr_dev(div10, "class", "flex items-start");
				add_location(div10, file$c, 85, 6, 3146);
				attr_dev(input4, "id", "block-featuring");
				attr_dev(input4, "type", "checkbox");
				input4.checked = input4_checked_value = /*options*/ ctx[0].blockFeaturing;
				attr_dev(input4, "class", "focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300 rounded");
				add_location(input4, file$c, 107, 10, 3937);
				attr_dev(div11, "class", "flex items-center h-5");
				add_location(div11, file$c, 106, 8, 3891);
				attr_dev(label4, "for", "block-featuring");
				attr_dev(label4, "class", "font-medium text-gray-700");
				add_location(label4, file$c, 116, 10, 4274);
				attr_dev(p3, "class", "text-gray-500");
				add_location(p3, file$c, 119, 10, 4395);
				attr_dev(div12, "class", "ml-3 text-sm");
				add_location(div12, file$c, 115, 8, 4237);
				attr_dev(div13, "class", "flex items-start");
				add_location(div13, file$c, 105, 6, 3852);
				attr_dev(input5, "id", "block-songwriter-only");
				attr_dev(input5, "type", "checkbox");
				input5.checked = input5_checked_value = /*options*/ ctx[0].blockSongwriterOnly;
				attr_dev(input5, "class", "focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300 rounded");
				add_location(input5, file$c, 127, 10, 4661);
				attr_dev(div14, "class", "flex items-center h-5");
				add_location(div14, file$c, 126, 8, 4615);
				attr_dev(label5, "for", "block-songwriter-only");
				attr_dev(label5, "class", "font-medium text-gray-700");
				add_location(label5, file$c, 136, 10, 5014);
				attr_dev(p4, "class", "text-gray-500");
				add_location(p4, file$c, 139, 10, 5155);
				attr_dev(div15, "class", "ml-3 text-sm");
				add_location(div15, file$c, 135, 8, 4977);
				attr_dev(div16, "class", "flex items-start");
				add_location(div16, file$c, 125, 6, 4576);
				attr_dev(div17, "class", "mt-4 space-y-4");
				add_location(div17, file$c, 84, 4, 3111);
				add_location(div18, file$c, 79, 2, 2861);
				attr_dev(div19, "class", "space-y-6");
				add_location(div19, file$c, 14, 0, 357);
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				insert_dev(target, div19, anchor);
				append_dev(div19, div7);
				append_dev(div7, h40);
				append_dev(div7, t1);
				append_dev(div7, p0);
				append_dev(div7, t3);
				append_dev(div7, fieldset);
				append_dev(fieldset, legend);
				append_dev(fieldset, t5);
				append_dev(fieldset, div6);
				append_dev(div6, div0);
				append_dev(div0, input0);
				append_dev(div0, t6);
				append_dev(div0, label0);
				append_dev(div6, t8);
				append_dev(div6, div1);
				append_dev(div6, t10);
				append_dev(div6, div2);
				append_dev(div2, input1);
				append_dev(div2, t11);
				append_dev(div2, label1);
				append_dev(div6, t13);
				append_dev(div6, div3);
				append_dev(div6, t15);
				append_dev(div6, div4);
				append_dev(div4, input2);
				append_dev(div4, t16);
				append_dev(div4, label2);
				append_dev(div6, t18);
				append_dev(div6, div5);
				append_dev(div19, t20);
				append_dev(div19, div18);
				append_dev(div18, h41);
				append_dev(div18, t22);
				append_dev(div18, p1);
				append_dev(div18, t24);
				append_dev(div18, div17);
				append_dev(div17, div10);
				append_dev(div10, div8);
				append_dev(div8, input3);
				append_dev(div10, t25);
				append_dev(div10, div9);
				append_dev(div9, label3);
				append_dev(div9, t27);
				append_dev(div9, p2);
				append_dev(div17, t29);
				append_dev(div17, div13);
				append_dev(div13, div11);
				append_dev(div11, input4);
				append_dev(div13, t30);
				append_dev(div13, div12);
				append_dev(div12, label4);
				append_dev(div12, t32);
				append_dev(div12, p3);
				append_dev(div17, t34);
				append_dev(div17, div16);
				append_dev(div16, div14);
				append_dev(div14, input5);
				append_dev(div16, t35);
				append_dev(div16, div15);
				append_dev(div15, label5);
				append_dev(div15, t37);
				append_dev(div15, p4);
				append_dev(div19, t39);
				if (if_block) if_block.m(div19, null);

				if (!mounted) {
					dispose = [
						listen_dev(input0, "change", /*change_handler*/ ctx[4], false, false, false, false),
						listen_dev(input1, "change", /*change_handler_1*/ ctx[5], false, false, false, false),
						listen_dev(input2, "change", /*change_handler_2*/ ctx[6], false, false, false, false),
						listen_dev(input3, "change", /*change_handler_3*/ ctx[7], false, false, false, false),
						listen_dev(input4, "change", /*change_handler_4*/ ctx[8], false, false, false, false),
						listen_dev(input5, "change", /*change_handler_5*/ ctx[9], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, [dirty]) {
				if (dirty & /*options*/ 1 && input0_checked_value !== (input0_checked_value = /*options*/ ctx[0].aggressiveness === 'conservative')) {
					prop_dev(input0, "checked", input0_checked_value);
				}

				if (dirty & /*options*/ 1 && input1_checked_value !== (input1_checked_value = /*options*/ ctx[0].aggressiveness === 'moderate')) {
					prop_dev(input1, "checked", input1_checked_value);
				}

				if (dirty & /*options*/ 1 && input2_checked_value !== (input2_checked_value = /*options*/ ctx[0].aggressiveness === 'aggressive')) {
					prop_dev(input2, "checked", input2_checked_value);
				}

				if (dirty & /*options*/ 1 && input3_checked_value !== (input3_checked_value = /*options*/ ctx[0].blockCollabs)) {
					prop_dev(input3, "checked", input3_checked_value);
				}

				if (dirty & /*options*/ 1 && input4_checked_value !== (input4_checked_value = /*options*/ ctx[0].blockFeaturing)) {
					prop_dev(input4, "checked", input4_checked_value);
				}

				if (dirty & /*options*/ 1 && input5_checked_value !== (input5_checked_value = /*options*/ ctx[0].blockSongwriterOnly)) {
					prop_dev(input5, "checked", input5_checked_value);
				}

				if (/*options*/ ctx[0].aggressiveness === 'aggressive' || /*options*/ ctx[0].blockSongwriterOnly) {
					if (if_block) ; else {
						if_block = create_if_block$c(ctx);
						if_block.c();
						if_block.m(div19, null);
					}
				} else if (if_block) {
					if_block.d(1);
					if_block = null;
				}
			},
			i: noop,
			o: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div19);
				}

				if (if_block) if_block.d();
				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_fragment$c.name,
			type: "component",
			source: "",
			ctx
		});

		return block;
	}

	function instance$c($$self, $$props, $$invalidate) {
		let options;
		let $enforcementStore;
		validate_store(enforcementStore, 'enforcementStore');
		component_subscribe($$self, enforcementStore, $$value => $$invalidate(3, $enforcementStore = $$value));
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('EnforcementOptions', slots, []);

		function updateAggressiveness(level) {
			enforcementActions.updateOptions({ aggressiveness: level });
		}

		function toggleOption(option) {
			enforcementActions.updateOptions({ [option]: !options[option] });
		}

		const writable_props = [];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<EnforcementOptions> was created with unknown prop '${key}'`);
		});

		const change_handler = () => updateAggressiveness('conservative');
		const change_handler_1 = () => updateAggressiveness('moderate');
		const change_handler_2 = () => updateAggressiveness('aggressive');
		const change_handler_3 = () => toggleOption('blockCollabs');
		const change_handler_4 = () => toggleOption('blockFeaturing');
		const change_handler_5 = () => toggleOption('blockSongwriterOnly');

		$$self.$capture_state = () => ({
			enforcementActions,
			enforcementStore,
			updateAggressiveness,
			toggleOption,
			options,
			$enforcementStore
		});

		$$self.$inject_state = $$props => {
			if ('options' in $$props) $$invalidate(0, options = $$props.options);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		$$self.$$.update = () => {
			if ($$self.$$.dirty & /*$enforcementStore*/ 8) {
				$$invalidate(0, options = $enforcementStore.options);
			}
		};

		return [
			options,
			updateAggressiveness,
			toggleOption,
			$enforcementStore,
			change_handler,
			change_handler_1,
			change_handler_2,
			change_handler_3,
			change_handler_4,
			change_handler_5
		];
	}

	class EnforcementOptions extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance$c, create_fragment$c, safe_not_equal, {});

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "EnforcementOptions",
				options,
				id: create_fragment$c.name
			});
		}
	}

	/* src/lib/components/EnforcementPreview.svelte generated by Svelte v4.2.20 */

	const { Object: Object_1 } = globals;
	const file$b = "src/lib/components/EnforcementPreview.svelte";

	function get_each_context$5(ctx, list, i) {
		const child_ctx = ctx.slice();
		child_ctx[2] = list[i][0];
		child_ctx[3] = list[i][1];
		return child_ctx;
	}

	function get_each_context_1$1(ctx, list, i) {
		const child_ctx = ctx.slice();
		child_ctx[6] = list[i][0];
		child_ctx[7] = list[i][1];
		return child_ctx;
	}

	// (194:0) {:else}
	function create_else_block$b(ctx) {
		let div;
		let svg;
		let path;
		let t0;
		let h3;
		let t2;
		let p;

		const block = {
			c: function create() {
				div = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				h3 = element("h3");
				h3.textContent = "No enforcement plan";
				t2 = space();
				p = element("p");
				p.textContent = "Create a plan to see the preview.";
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2");
				add_location(path, file$b, 200, 6, 9233);
				attr_dev(svg, "class", "mx-auto h-12 w-12 text-gray-400");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$b, 199, 4, 9127);
				attr_dev(h3, "class", "mt-2 text-sm font-medium text-gray-900");
				add_location(h3, file$b, 202, 4, 9452);
				attr_dev(p, "class", "mt-1 text-sm text-gray-500");
				add_location(p, file$b, 203, 4, 9532);
				attr_dev(div, "class", "text-center py-6");
				add_location(div, file$b, 198, 2, 9092);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, svg);
				append_dev(svg, path);
				append_dev(div, t0);
				append_dev(div, h3);
				append_dev(div, t2);
				append_dev(div, p);
			},
			p: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block$b.name,
			type: "else",
			source: "(194:0) {:else}",
			ctx
		});

		return block;
	}

	// (33:0) {#if plan}
	function create_if_block$b(ctx) {
		let div11;
		let div5;
		let div4;
		let div0;
		let h4;
		let t1;
		let p;
		let t2;
		let t3_value = formatDuration(/*plan*/ ctx[0].estimatedDuration) + "";
		let t3;
		let t4;
		let t5;
		let div3;
		let div1;
		let t7;
		let div2;
		let t8_value = /*plan*/ ctx[0].planId.slice(0, 8) + "";
		let t8;
		let t9;
		let t10;
		let t11;
		let div10;
		let div9;
		let div6;
		let svg;
		let path;
		let t12;
		let div8;
		let h3;
		let t14;
		let div7;
		let ul;
		let li0;
		let t16;
		let li1;
		let t18;
		let li2;
		let t20;
		let li3;
		let if_block = /*plan*/ ctx[0].resumable && create_if_block_8$3(ctx);
		let each_value = ensure_array_like_dev(Object.entries(/*plan*/ ctx[0].impact));
		let each_blocks = [];

		for (let i = 0; i < each_value.length; i += 1) {
			each_blocks[i] = create_each_block$5(get_each_context$5(ctx, each_value, i));
		}

		const block = {
			c: function create() {
				div11 = element("div");
				div5 = element("div");
				div4 = element("div");
				div0 = element("div");
				h4 = element("h4");
				h4.textContent = "Plan Summary";
				t1 = space();
				p = element("p");
				t2 = text("Estimated duration: ");
				t3 = text(t3_value);
				t4 = space();
				if (if_block) if_block.c();
				t5 = space();
				div3 = element("div");
				div1 = element("div");
				div1.textContent = "Plan ID";
				t7 = space();
				div2 = element("div");
				t8 = text(t8_value);
				t9 = text("...");
				t10 = space();

				for (let i = 0; i < each_blocks.length; i += 1) {
					each_blocks[i].c();
				}

				t11 = space();
				div10 = element("div");
				div9 = element("div");
				div6 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t12 = space();
				div8 = element("div");
				h3 = element("h3");
				h3.textContent = "Before You Execute";
				t14 = space();
				div7 = element("div");
				ul = element("ul");
				li0 = element("li");
				li0.textContent = "This is a preview - no changes have been made yet";
				t16 = space();
				li1 = element("li");
				li1.textContent = "Execution will modify your actual music library";
				t18 = space();
				li2 = element("li");
				li2.textContent = "Some actions may not be reversible depending on platform limitations";
				t20 = space();
				li3 = element("li");
				li3.textContent = "The process can be interrupted and resumed if needed";
				attr_dev(h4, "class", "text-sm font-medium text-gray-900");
				add_location(h4, file$b, 42, 10, 1841);
				attr_dev(p, "class", "text-sm text-gray-500");
				add_location(p, file$b, 43, 10, 1915);
				add_location(div0, file$b, 41, 8, 1825);
				attr_dev(div1, "class", "text-sm font-medium text-gray-900");
				add_location(div1, file$b, 51, 10, 2187);
				attr_dev(div2, "class", "text-xs text-gray-500 font-mono");
				add_location(div2, file$b, 52, 10, 2258);
				attr_dev(div3, "class", "text-right");
				add_location(div3, file$b, 50, 8, 2152);
				attr_dev(div4, "class", "flex items-center justify-between");
				add_location(div4, file$b, 40, 6, 1769);
				attr_dev(div5, "class", "bg-gray-50 rounded-lg p-4");
				add_location(div5, file$b, 39, 4, 1723);
				attr_dev(path, "fill-rule", "evenodd");
				attr_dev(path, "d", "M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z");
				attr_dev(path, "clip-rule", "evenodd");
				add_location(path, file$b, 178, 12, 8234);
				attr_dev(svg, "class", "h-5 w-5 text-blue-400");
				attr_dev(svg, "viewBox", "0 0 20 20");
				attr_dev(svg, "fill", "currentColor");
				add_location(svg, file$b, 177, 10, 8146);
				attr_dev(div6, "class", "flex-shrink-0");
				add_location(div6, file$b, 176, 8, 8108);
				attr_dev(h3, "class", "text-sm font-medium text-blue-800");
				add_location(h3, file$b, 182, 10, 8485);
				add_location(li0, file$b, 187, 14, 8701);
				add_location(li1, file$b, 188, 14, 8774);
				add_location(li2, file$b, 189, 14, 8845);
				add_location(li3, file$b, 190, 14, 8937);
				attr_dev(ul, "class", "list-disc list-inside space-y-1");
				add_location(ul, file$b, 186, 12, 8642);
				attr_dev(div7, "class", "mt-2 text-sm text-blue-700");
				add_location(div7, file$b, 185, 10, 8589);
				attr_dev(div8, "class", "ml-3");
				add_location(div8, file$b, 181, 8, 8456);
				attr_dev(div9, "class", "flex");
				add_location(div9, file$b, 175, 6, 8081);
				attr_dev(div10, "class", "bg-blue-50 border border-blue-200 rounded-md p-4");
				add_location(div10, file$b, 174, 4, 8012);
				attr_dev(div11, "class", "space-y-6");
				add_location(div11, file$b, 37, 2, 1669);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div11, anchor);
				append_dev(div11, div5);
				append_dev(div5, div4);
				append_dev(div4, div0);
				append_dev(div0, h4);
				append_dev(div0, t1);
				append_dev(div0, p);
				append_dev(p, t2);
				append_dev(p, t3);
				append_dev(p, t4);
				if (if_block) if_block.m(p, null);
				append_dev(div4, t5);
				append_dev(div4, div3);
				append_dev(div3, div1);
				append_dev(div3, t7);
				append_dev(div3, div2);
				append_dev(div2, t8);
				append_dev(div2, t9);
				append_dev(div11, t10);

				for (let i = 0; i < each_blocks.length; i += 1) {
					if (each_blocks[i]) {
						each_blocks[i].m(div11, null);
					}
				}

				append_dev(div11, t11);
				append_dev(div11, div10);
				append_dev(div10, div9);
				append_dev(div9, div6);
				append_dev(div6, svg);
				append_dev(svg, path);
				append_dev(div9, t12);
				append_dev(div9, div8);
				append_dev(div8, h3);
				append_dev(div8, t14);
				append_dev(div8, div7);
				append_dev(div7, ul);
				append_dev(ul, li0);
				append_dev(ul, t16);
				append_dev(ul, li1);
				append_dev(ul, t18);
				append_dev(ul, li2);
				append_dev(ul, t20);
				append_dev(ul, li3);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*plan*/ 1 && t3_value !== (t3_value = formatDuration(/*plan*/ ctx[0].estimatedDuration) + "")) set_data_dev(t3, t3_value);

				if (/*plan*/ ctx[0].resumable) {
					if (if_block) ; else {
						if_block = create_if_block_8$3(ctx);
						if_block.c();
						if_block.m(p, null);
					}
				} else if (if_block) {
					if_block.d(1);
					if_block = null;
				}

				if (dirty & /*plan*/ 1 && t8_value !== (t8_value = /*plan*/ ctx[0].planId.slice(0, 8) + "")) set_data_dev(t8, t8_value);

				if (dirty & /*Object, plan, getCapabilityColor, getProviderIcon*/ 1) {
					each_value = ensure_array_like_dev(Object.entries(/*plan*/ ctx[0].impact));
					let i;

					for (i = 0; i < each_value.length; i += 1) {
						const child_ctx = get_each_context$5(ctx, each_value, i);

						if (each_blocks[i]) {
							each_blocks[i].p(child_ctx, dirty);
						} else {
							each_blocks[i] = create_each_block$5(child_ctx);
							each_blocks[i].c();
							each_blocks[i].m(div11, t11);
						}
					}

					for (; i < each_blocks.length; i += 1) {
						each_blocks[i].d(1);
					}

					each_blocks.length = each_value.length;
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div11);
				}

				if (if_block) if_block.d();
				destroy_each(each_blocks, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block$b.name,
			type: "if",
			source: "(33:0) {#if plan}",
			ctx
		});

		return block;
	}

	// (42:12) {#if plan.resumable}
	function create_if_block_8$3(ctx) {
		let t;

		const block = {
			c: function create() {
				t = text("• Resumable if interrupted");
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
			id: create_if_block_8$3.name,
			type: "if",
			source: "(42:12) {#if plan.resumable}",
			ctx
		});

		return block;
	}

	// (71:10) {#if impact.likedSongs}
	function create_if_block_6$7(ctx) {
		let div3;
		let div2;
		let div0;
		let svg;
		let path;
		let t0;
		let div1;
		let p0;
		let t2;
		let p1;
		let t3_value = /*impact*/ ctx[3].likedSongs.toRemove + "";
		let t3;
		let t4;
		let if_block = /*impact*/ ctx[3].likedSongs.collabsFound > 0 && create_if_block_7$5(ctx);

		const block = {
			c: function create() {
				div3 = element("div");
				div2 = element("div");
				div0 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				div1 = element("div");
				p0 = element("p");
				p0.textContent = "Liked Songs";
				t2 = space();
				p1 = element("p");
				t3 = text(t3_value);
				t4 = text(" to remove\n                    ");
				if (if_block) if_block.c();
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z");
				add_location(path, file$b, 79, 20, 3457);
				attr_dev(svg, "class", "h-6 w-6 text-red-400");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$b, 78, 18, 3348);
				attr_dev(div0, "class", "flex-shrink-0");
				add_location(div0, file$b, 77, 16, 3302);
				attr_dev(p0, "class", "text-sm font-medium text-gray-900");
				add_location(p0, file$b, 83, 18, 3759);
				attr_dev(p1, "class", "text-sm text-gray-500");
				add_location(p1, file$b, 84, 18, 3838);
				attr_dev(div1, "class", "ml-3");
				add_location(div1, file$b, 82, 16, 3722);
				attr_dev(div2, "class", "flex items-center");
				add_location(div2, file$b, 76, 14, 3254);
				attr_dev(div3, "class", "bg-white border border-gray-200 rounded-lg p-4");
				add_location(div3, file$b, 75, 12, 3179);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div3, anchor);
				append_dev(div3, div2);
				append_dev(div2, div0);
				append_dev(div0, svg);
				append_dev(svg, path);
				append_dev(div2, t0);
				append_dev(div2, div1);
				append_dev(div1, p0);
				append_dev(div1, t2);
				append_dev(div1, p1);
				append_dev(p1, t3);
				append_dev(p1, t4);
				if (if_block) if_block.m(p1, null);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*plan*/ 1 && t3_value !== (t3_value = /*impact*/ ctx[3].likedSongs.toRemove + "")) set_data_dev(t3, t3_value);

				if (/*impact*/ ctx[3].likedSongs.collabsFound > 0) {
					if (if_block) {
						if_block.p(ctx, dirty);
					} else {
						if_block = create_if_block_7$5(ctx);
						if_block.c();
						if_block.m(p1, null);
					}
				} else if (if_block) {
					if_block.d(1);
					if_block = null;
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div3);
				}

				if (if_block) if_block.d();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_6$7.name,
			type: "if",
			source: "(71:10) {#if impact.likedSongs}",
			ctx
		});

		return block;
	}

	// (83:20) {#if impact.likedSongs.collabsFound > 0}
	function create_if_block_7$5(ctx) {
		let br;
		let span;
		let t0;
		let t1_value = /*impact*/ ctx[3].likedSongs.collabsFound + "";
		let t1;
		let t2;

		const block = {
			c: function create() {
				br = element("br");
				span = element("span");
				t0 = text("(");
				t1 = text(t1_value);
				t2 = text(" collaborations)");
				add_location(br, file$b, 87, 22, 4014);
				attr_dev(span, "class", "text-xs");
				add_location(span, file$b, 87, 28, 4020);
			},
			m: function mount(target, anchor) {
				insert_dev(target, br, anchor);
				insert_dev(target, span, anchor);
				append_dev(span, t0);
				append_dev(span, t1);
				append_dev(span, t2);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*plan*/ 1 && t1_value !== (t1_value = /*impact*/ ctx[3].likedSongs.collabsFound + "")) set_data_dev(t1, t1_value);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(br);
					detach_dev(span);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_7$5.name,
			type: "if",
			source: "(83:20) {#if impact.likedSongs.collabsFound > 0}",
			ctx
		});

		return block;
	}

	// (93:10) {#if impact.playlists}
	function create_if_block_4$7(ctx) {
		let div3;
		let div2;
		let div0;
		let svg;
		let path;
		let t0;
		let div1;
		let p0;
		let t2;
		let p1;
		let t3_value = /*impact*/ ctx[3].playlists.toScrub + "";
		let t3;
		let t4;
		let br;
		let span;
		let t5_value = /*impact*/ ctx[3].playlists.tracksToRemove + "";
		let t5;
		let t6;
		let t7;
		let if_block = /*impact*/ ctx[3].playlists.featuringFound > 0 && create_if_block_5$7(ctx);

		const block = {
			c: function create() {
				div3 = element("div");
				div2 = element("div");
				div0 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				div1 = element("div");
				p0 = element("p");
				p0.textContent = "Playlists";
				t2 = space();
				p1 = element("p");
				t3 = text(t3_value);
				t4 = text(" playlists affected\n                    ");
				br = element("br");
				span = element("span");
				t5 = text(t5_value);
				t6 = text(" tracks to remove");
				t7 = space();
				if (if_block) if_block.c();
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3");
				add_location(path, file$b, 101, 20, 4581);
				attr_dev(svg, "class", "h-6 w-6 text-blue-400");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$b, 100, 18, 4471);
				attr_dev(div0, "class", "flex-shrink-0");
				add_location(div0, file$b, 99, 16, 4425);
				attr_dev(p0, "class", "text-sm font-medium text-gray-900");
				add_location(p0, file$b, 105, 18, 4907);
				add_location(br, file$b, 108, 20, 5104);
				attr_dev(span, "class", "text-xs");
				add_location(span, file$b, 108, 26, 5110);
				attr_dev(p1, "class", "text-sm text-gray-500");
				add_location(p1, file$b, 106, 18, 4984);
				attr_dev(div1, "class", "ml-3");
				add_location(div1, file$b, 104, 16, 4870);
				attr_dev(div2, "class", "flex items-center");
				add_location(div2, file$b, 98, 14, 4377);
				attr_dev(div3, "class", "bg-white border border-gray-200 rounded-lg p-4");
				add_location(div3, file$b, 97, 12, 4302);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div3, anchor);
				append_dev(div3, div2);
				append_dev(div2, div0);
				append_dev(div0, svg);
				append_dev(svg, path);
				append_dev(div2, t0);
				append_dev(div2, div1);
				append_dev(div1, p0);
				append_dev(div1, t2);
				append_dev(div1, p1);
				append_dev(p1, t3);
				append_dev(p1, t4);
				append_dev(p1, br);
				append_dev(p1, span);
				append_dev(span, t5);
				append_dev(span, t6);
				append_dev(p1, t7);
				if (if_block) if_block.m(p1, null);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*plan*/ 1 && t3_value !== (t3_value = /*impact*/ ctx[3].playlists.toScrub + "")) set_data_dev(t3, t3_value);
				if (dirty & /*plan*/ 1 && t5_value !== (t5_value = /*impact*/ ctx[3].playlists.tracksToRemove + "")) set_data_dev(t5, t5_value);

				if (/*impact*/ ctx[3].playlists.featuringFound > 0) {
					if (if_block) {
						if_block.p(ctx, dirty);
					} else {
						if_block = create_if_block_5$7(ctx);
						if_block.c();
						if_block.m(p1, null);
					}
				} else if (if_block) {
					if_block.d(1);
					if_block = null;
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div3);
				}

				if (if_block) if_block.d();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_4$7.name,
			type: "if",
			source: "(93:10) {#if impact.playlists}",
			ctx
		});

		return block;
	}

	// (106:20) {#if impact.playlists.featuringFound > 0}
	function create_if_block_5$7(ctx) {
		let br;
		let span;
		let t0;
		let t1_value = /*impact*/ ctx[3].playlists.featuringFound + "";
		let t1;
		let t2;

		const block = {
			c: function create() {
				br = element("br");
				span = element("span");
				t0 = text("(");
				t1 = text(t1_value);
				t2 = text(" featuring)");
				add_location(br, file$b, 110, 22, 5274);
				attr_dev(span, "class", "text-xs");
				add_location(span, file$b, 110, 28, 5280);
			},
			m: function mount(target, anchor) {
				insert_dev(target, br, anchor);
				insert_dev(target, span, anchor);
				append_dev(span, t0);
				append_dev(span, t1);
				append_dev(span, t2);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*plan*/ 1 && t1_value !== (t1_value = /*impact*/ ctx[3].playlists.featuringFound + "")) set_data_dev(t1, t1_value);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(br);
					detach_dev(span);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_5$7.name,
			type: "if",
			source: "(106:20) {#if impact.playlists.featuringFound > 0}",
			ctx
		});

		return block;
	}

	// (116:10) {#if impact.following}
	function create_if_block_3$7(ctx) {
		let div3;
		let div2;
		let div0;
		let svg;
		let path;
		let t0;
		let div1;
		let p0;
		let t2;
		let p1;
		let t3_value = /*impact*/ ctx[3].following.toUnfollow + "";
		let t3;
		let t4;

		const block = {
			c: function create() {
				div3 = element("div");
				div2 = element("div");
				div0 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				div1 = element("div");
				p0 = element("p");
				p0.textContent = "Following";
				t2 = space();
				p1 = element("p");
				t3 = text(t3_value);
				t4 = text(" to unfollow");
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z");
				add_location(path, file$b, 124, 20, 5839);
				attr_dev(svg, "class", "h-6 w-6 text-purple-400");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$b, 123, 18, 5727);
				attr_dev(div0, "class", "flex-shrink-0");
				add_location(div0, file$b, 122, 16, 5681);
				attr_dev(p0, "class", "text-sm font-medium text-gray-900");
				add_location(p0, file$b, 128, 18, 6085);
				attr_dev(p1, "class", "text-sm text-gray-500");
				add_location(p1, file$b, 129, 18, 6162);
				attr_dev(div1, "class", "ml-3");
				add_location(div1, file$b, 127, 16, 6048);
				attr_dev(div2, "class", "flex items-center");
				add_location(div2, file$b, 121, 14, 5633);
				attr_dev(div3, "class", "bg-white border border-gray-200 rounded-lg p-4");
				add_location(div3, file$b, 120, 12, 5558);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div3, anchor);
				append_dev(div3, div2);
				append_dev(div2, div0);
				append_dev(div0, svg);
				append_dev(svg, path);
				append_dev(div2, t0);
				append_dev(div2, div1);
				append_dev(div1, p0);
				append_dev(div1, t2);
				append_dev(div1, p1);
				append_dev(p1, t3);
				append_dev(p1, t4);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*plan*/ 1 && t3_value !== (t3_value = /*impact*/ ctx[3].following.toUnfollow + "")) set_data_dev(t3, t3_value);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div3);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_3$7.name,
			type: "if",
			source: "(116:10) {#if impact.following}",
			ctx
		});

		return block;
	}

	// (135:10) {#if impact.radioSeeds}
	function create_if_block_2$8(ctx) {
		let div3;
		let div2;
		let div0;
		let svg;
		let path;
		let t0;
		let div1;
		let p0;
		let t2;
		let p1;
		let t3_value = /*impact*/ ctx[3].radioSeeds.toFilter + "";
		let t3;
		let t4;

		const block = {
			c: function create() {
				div3 = element("div");
				div2 = element("div");
				div0 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				div1 = element("div");
				p0 = element("p");
				p0.textContent = "Radio Seeds";
				t2 = space();
				p1 = element("p");
				t3 = text(t3_value);
				t4 = text(" to filter");
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M7 4V2a1 1 0 011-1h8a1 1 0 011 1v2m-9 0h10m-10 0a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V6a2 2 0 00-2-2M7 4h10");
				add_location(path, file$b, 143, 20, 6719);
				attr_dev(svg, "class", "h-6 w-6 text-orange-400");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$b, 142, 18, 6607);
				attr_dev(div0, "class", "flex-shrink-0");
				add_location(div0, file$b, 141, 16, 6561);
				attr_dev(p0, "class", "text-sm font-medium text-gray-900");
				add_location(p0, file$b, 147, 18, 7011);
				attr_dev(p1, "class", "text-sm text-gray-500");
				add_location(p1, file$b, 148, 18, 7090);
				attr_dev(div1, "class", "ml-3");
				add_location(div1, file$b, 146, 16, 6974);
				attr_dev(div2, "class", "flex items-center");
				add_location(div2, file$b, 140, 14, 6513);
				attr_dev(div3, "class", "bg-white border border-gray-200 rounded-lg p-4");
				add_location(div3, file$b, 139, 12, 6438);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div3, anchor);
				append_dev(div3, div2);
				append_dev(div2, div0);
				append_dev(div0, svg);
				append_dev(svg, path);
				append_dev(div2, t0);
				append_dev(div2, div1);
				append_dev(div1, p0);
				append_dev(div1, t2);
				append_dev(div1, p1);
				append_dev(p1, t3);
				append_dev(p1, t4);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*plan*/ 1 && t3_value !== (t3_value = /*impact*/ ctx[3].radioSeeds.toFilter + "")) set_data_dev(t3, t3_value);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div3);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_2$8.name,
			type: "if",
			source: "(135:10) {#if impact.radioSeeds}",
			ctx
		});

		return block;
	}

	// (155:8) {#if plan.capabilities[provider]}
	function create_if_block_1$b(ctx) {
		let div1;
		let h5;
		let t1;
		let div0;
		let each_value_1 = ensure_array_like_dev(Object.entries(/*plan*/ ctx[0].capabilities[/*provider*/ ctx[2]]));
		let each_blocks = [];

		for (let i = 0; i < each_value_1.length; i += 1) {
			each_blocks[i] = create_each_block_1$1(get_each_context_1$1(ctx, each_value_1, i));
		}

		const block = {
			c: function create() {
				div1 = element("div");
				h5 = element("h5");
				h5.textContent = "Platform Capabilities";
				t1 = space();
				div0 = element("div");

				for (let i = 0; i < each_blocks.length; i += 1) {
					each_blocks[i].c();
				}

				attr_dev(h5, "class", "text-sm font-medium text-gray-900 mb-2");
				add_location(h5, file$b, 160, 12, 7444);
				attr_dev(div0, "class", "flex flex-wrap gap-2");
				add_location(div0, file$b, 161, 12, 7534);
				attr_dev(div1, "class", "mt-4 pt-4 border-t border-gray-200");
				add_location(div1, file$b, 159, 10, 7383);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div1, anchor);
				append_dev(div1, h5);
				append_dev(div1, t1);
				append_dev(div1, div0);

				for (let i = 0; i < each_blocks.length; i += 1) {
					if (each_blocks[i]) {
						each_blocks[i].m(div0, null);
					}
				}
			},
			p: function update(ctx, dirty) {
				if (dirty & /*getCapabilityColor, Object, plan*/ 1) {
					each_value_1 = ensure_array_like_dev(Object.entries(/*plan*/ ctx[0].capabilities[/*provider*/ ctx[2]]));
					let i;

					for (i = 0; i < each_value_1.length; i += 1) {
						const child_ctx = get_each_context_1$1(ctx, each_value_1, i);

						if (each_blocks[i]) {
							each_blocks[i].p(child_ctx, dirty);
						} else {
							each_blocks[i] = create_each_block_1$1(child_ctx);
							each_blocks[i].c();
							each_blocks[i].m(div0, null);
						}
					}

					for (; i < each_blocks.length; i += 1) {
						each_blocks[i].d(1);
					}

					each_blocks.length = each_value_1.length;
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div1);
				}

				destroy_each(each_blocks, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_1$b.name,
			type: "if",
			source: "(155:8) {#if plan.capabilities[provider]}",
			ctx
		});

		return block;
	}

	// (159:14) {#each Object.entries(plan.capabilities[provider]) as [capability, support]}
	function create_each_block_1$1(ctx) {
		let span;
		let t0_value = /*capability*/ ctx[6].replace(/_/g, ' ').toLowerCase() + "";
		let t0;
		let t1;
		let span_class_value;

		const block = {
			c: function create() {
				span = element("span");
				t0 = text(t0_value);
				t1 = space();
				attr_dev(span, "class", span_class_value = "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium " + getCapabilityColor(/*support*/ ctx[7]));
				add_location(span, file$b, 163, 16, 7676);
			},
			m: function mount(target, anchor) {
				insert_dev(target, span, anchor);
				append_dev(span, t0);
				append_dev(span, t1);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*plan*/ 1 && t0_value !== (t0_value = /*capability*/ ctx[6].replace(/_/g, ' ').toLowerCase() + "")) set_data_dev(t0, t0_value);

				if (dirty & /*plan*/ 1 && span_class_value !== (span_class_value = "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium " + getCapabilityColor(/*support*/ ctx[7]))) {
					attr_dev(span, "class", span_class_value);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(span);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_each_block_1$1.name,
			type: "each",
			source: "(159:14) {#each Object.entries(plan.capabilities[provider]) as [capability, support]}",
			ctx
		});

		return block;
	}

	// (55:4) {#each Object.entries(plan.impact) as [provider, impact]}
	function create_each_block$5(ctx) {
		let div4;
		let div2;
		let div0;
		let svg;
		let path;
		let path_d_value;
		let t0;
		let div1;
		let h4;
		let t1_value = /*provider*/ ctx[2] + "";
		let t1;
		let t2;
		let p;
		let t3;
		let t4_value = /*provider*/ ctx[2] + "";
		let t4;
		let t5;
		let t6;
		let div3;
		let t7;
		let t8;
		let t9;
		let t10;
		let t11;
		let if_block0 = /*impact*/ ctx[3].likedSongs && create_if_block_6$7(ctx);
		let if_block1 = /*impact*/ ctx[3].playlists && create_if_block_4$7(ctx);
		let if_block2 = /*impact*/ ctx[3].following && create_if_block_3$7(ctx);
		let if_block3 = /*impact*/ ctx[3].radioSeeds && create_if_block_2$8(ctx);
		let if_block4 = /*plan*/ ctx[0].capabilities[/*provider*/ ctx[2]] && create_if_block_1$b(ctx);

		const block = {
			c: function create() {
				div4 = element("div");
				div2 = element("div");
				div0 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				div1 = element("div");
				h4 = element("h4");
				t1 = text(t1_value);
				t2 = space();
				p = element("p");
				t3 = text("Impact preview for your ");
				t4 = text(t4_value);
				t5 = text(" library");
				t6 = space();
				div3 = element("div");
				if (if_block0) if_block0.c();
				t7 = space();
				if (if_block1) if_block1.c();
				t8 = space();
				if (if_block2) if_block2.c();
				t9 = space();
				if (if_block3) if_block3.c();
				t10 = space();
				if (if_block4) if_block4.c();
				t11 = space();
				attr_dev(path, "d", path_d_value = getProviderIcon(/*provider*/ ctx[2]));
				add_location(path, file$b, 63, 14, 2713);
				attr_dev(svg, "class", "h-8 w-8 text-green-500");
				attr_dev(svg, "fill", "currentColor");
				attr_dev(svg, "viewBox", "0 0 24 24");
				add_location(svg, file$b, 62, 12, 2622);
				attr_dev(div0, "class", "flex-shrink-0");
				add_location(div0, file$b, 61, 10, 2582);
				attr_dev(h4, "class", "text-lg font-medium text-gray-900 capitalize");
				add_location(h4, file$b, 67, 12, 2829);
				attr_dev(p, "class", "text-sm text-gray-500");
				add_location(p, file$b, 68, 12, 2914);
				attr_dev(div1, "class", "ml-3");
				add_location(div1, file$b, 66, 10, 2798);
				attr_dev(div2, "class", "flex items-center mb-4");
				add_location(div2, file$b, 60, 8, 2535);
				attr_dev(div3, "class", "grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4");
				add_location(div3, file$b, 72, 8, 3035);
				attr_dev(div4, "class", "border border-gray-200 rounded-lg p-6");
				add_location(div4, file$b, 59, 6, 2475);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div4, anchor);
				append_dev(div4, div2);
				append_dev(div2, div0);
				append_dev(div0, svg);
				append_dev(svg, path);
				append_dev(div2, t0);
				append_dev(div2, div1);
				append_dev(div1, h4);
				append_dev(h4, t1);
				append_dev(div1, t2);
				append_dev(div1, p);
				append_dev(p, t3);
				append_dev(p, t4);
				append_dev(p, t5);
				append_dev(div4, t6);
				append_dev(div4, div3);
				if (if_block0) if_block0.m(div3, null);
				append_dev(div3, t7);
				if (if_block1) if_block1.m(div3, null);
				append_dev(div3, t8);
				if (if_block2) if_block2.m(div3, null);
				append_dev(div3, t9);
				if (if_block3) if_block3.m(div3, null);
				append_dev(div4, t10);
				if (if_block4) if_block4.m(div4, null);
				append_dev(div4, t11);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*plan*/ 1 && path_d_value !== (path_d_value = getProviderIcon(/*provider*/ ctx[2]))) {
					attr_dev(path, "d", path_d_value);
				}

				if (dirty & /*plan*/ 1 && t1_value !== (t1_value = /*provider*/ ctx[2] + "")) set_data_dev(t1, t1_value);
				if (dirty & /*plan*/ 1 && t4_value !== (t4_value = /*provider*/ ctx[2] + "")) set_data_dev(t4, t4_value);

				if (/*impact*/ ctx[3].likedSongs) {
					if (if_block0) {
						if_block0.p(ctx, dirty);
					} else {
						if_block0 = create_if_block_6$7(ctx);
						if_block0.c();
						if_block0.m(div3, t7);
					}
				} else if (if_block0) {
					if_block0.d(1);
					if_block0 = null;
				}

				if (/*impact*/ ctx[3].playlists) {
					if (if_block1) {
						if_block1.p(ctx, dirty);
					} else {
						if_block1 = create_if_block_4$7(ctx);
						if_block1.c();
						if_block1.m(div3, t8);
					}
				} else if (if_block1) {
					if_block1.d(1);
					if_block1 = null;
				}

				if (/*impact*/ ctx[3].following) {
					if (if_block2) {
						if_block2.p(ctx, dirty);
					} else {
						if_block2 = create_if_block_3$7(ctx);
						if_block2.c();
						if_block2.m(div3, t9);
					}
				} else if (if_block2) {
					if_block2.d(1);
					if_block2 = null;
				}

				if (/*impact*/ ctx[3].radioSeeds) {
					if (if_block3) {
						if_block3.p(ctx, dirty);
					} else {
						if_block3 = create_if_block_2$8(ctx);
						if_block3.c();
						if_block3.m(div3, null);
					}
				} else if (if_block3) {
					if_block3.d(1);
					if_block3 = null;
				}

				if (/*plan*/ ctx[0].capabilities[/*provider*/ ctx[2]]) {
					if (if_block4) {
						if_block4.p(ctx, dirty);
					} else {
						if_block4 = create_if_block_1$b(ctx);
						if_block4.c();
						if_block4.m(div4, t11);
					}
				} else if (if_block4) {
					if_block4.d(1);
					if_block4 = null;
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div4);
				}

				if (if_block0) if_block0.d();
				if (if_block1) if_block1.d();
				if (if_block2) if_block2.d();
				if (if_block3) if_block3.d();
				if (if_block4) if_block4.d();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_each_block$5.name,
			type: "each",
			source: "(55:4) {#each Object.entries(plan.impact) as [provider, impact]}",
			ctx
		});

		return block;
	}

	function create_fragment$b(ctx) {
		let if_block_anchor;

		function select_block_type(ctx, dirty) {
			if (/*plan*/ ctx[0]) return create_if_block$b;
			return create_else_block$b;
		}

		let current_block_type = select_block_type(ctx);
		let if_block = current_block_type(ctx);

		const block = {
			c: function create() {
				if_block.c();
				if_block_anchor = empty();
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				if_block.m(target, anchor);
				insert_dev(target, if_block_anchor, anchor);
			},
			p: function update(ctx, [dirty]) {
				if (current_block_type === (current_block_type = select_block_type(ctx)) && if_block) {
					if_block.p(ctx, dirty);
				} else {
					if_block.d(1);
					if_block = current_block_type(ctx);

					if (if_block) {
						if_block.c();
						if_block.m(if_block_anchor.parentNode, if_block_anchor);
					}
				}
			},
			i: noop,
			o: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(if_block_anchor);
				}

				if_block.d(detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_fragment$b.name,
			type: "component",
			source: "",
			ctx
		});

		return block;
	}

	function formatDuration(duration) {
		const seconds = parseInt(duration.replace('s', ''));
		if (seconds < 60) return `${seconds} seconds`;
		const minutes = Math.floor(seconds / 60);
		const remainingSeconds = seconds % 60;
		return `${minutes}m ${remainingSeconds}s`;
	}

	function getProviderIcon(provider) {
		switch (provider) {
			case 'spotify':
				return 'M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.42 1.56-.299.421-1.02.599-1.559.3z';
			default:
				return 'M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1';
		}
	}

	function getCapabilityColor(capability) {
		switch (capability) {
			case 'SUPPORTED':
				return 'text-green-600 bg-green-100';
			case 'LIMITED':
				return 'text-yellow-600 bg-yellow-100';
			case 'UNSUPPORTED':
				return 'text-red-600 bg-red-100';
			default:
				return 'text-gray-600 bg-gray-100';
		}
	}

	function instance$b($$self, $$props, $$invalidate) {
		let plan;
		let $enforcementStore;
		validate_store(enforcementStore, 'enforcementStore');
		component_subscribe($$self, enforcementStore, $$value => $$invalidate(1, $enforcementStore = $$value));
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('EnforcementPreview', slots, []);
		const writable_props = [];

		Object_1.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<EnforcementPreview> was created with unknown prop '${key}'`);
		});

		$$self.$capture_state = () => ({
			enforcementStore,
			formatDuration,
			getProviderIcon,
			getCapabilityColor,
			plan,
			$enforcementStore
		});

		$$self.$inject_state = $$props => {
			if ('plan' in $$props) $$invalidate(0, plan = $$props.plan);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		$$self.$$.update = () => {
			if ($$self.$$.dirty & /*$enforcementStore*/ 2) {
				$$invalidate(0, plan = $enforcementStore.currentPlan);
			}
		};

		return [plan, $enforcementStore];
	}

	class EnforcementPreview extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance$b, create_fragment$b, safe_not_equal, {});

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "EnforcementPreview",
				options,
				id: create_fragment$b.name
			});
		}
	}

	/* src/lib/components/EnforcementExecution.svelte generated by Svelte v4.2.20 */

	const file$a = "src/lib/components/EnforcementExecution.svelte";

	function get_each_context$4(ctx, list, i) {
		const child_ctx = ctx.slice();
		child_ctx[7] = list[i];
		return child_ctx;
	}

	// (150:2) {:else}
	function create_else_block$a(ctx) {
		let div3;
		let div2;
		let svg;
		let path;
		let t0;
		let h3;
		let t2;
		let p0;
		let t4;
		let div0;
		let button;
		let button_disabled_value;
		let t5;
		let div1;
		let p1;
		let t7;
		let p2;
		let mounted;
		let dispose;

		function select_block_type_1(ctx, dirty) {
			if (/*$enforcementStore*/ ctx[0].isExecuting) return create_if_block_6$6;
			return create_else_block_1$5;
		}

		let current_block_type = select_block_type_1(ctx);
		let if_block = current_block_type(ctx);

		const block = {
			c: function create() {
				div3 = element("div");
				div2 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				h3 = element("h3");
				h3.textContent = "Ready to Execute";
				t2 = space();
				p0 = element("p");
				p0.textContent = "Your enforcement plan is ready. Click execute to apply changes to your music library.";
				t4 = space();
				div0 = element("div");
				button = element("button");
				if_block.c();
				t5 = space();
				div1 = element("div");
				p1 = element("p");
				p1.textContent = "⚠️ This action will modify your music library";
				t7 = space();
				p2 = element("p");
				p2.textContent = "Some changes may not be reversible";
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M14.828 14.828a4 4 0 01-5.656 0M9 10h1.01M15 10h1.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z");
				add_location(path, file$a, 164, 10, 7092);
				attr_dev(svg, "class", "mx-auto h-12 w-12 text-indigo-400");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$a, 163, 8, 6980);
				attr_dev(h3, "class", "mt-2 text-lg font-medium text-gray-900");
				add_location(h3, file$a, 166, 8, 7279);
				attr_dev(p0, "class", "mt-1 text-sm text-gray-500");
				add_location(p0, file$a, 167, 8, 7360);
				button.disabled = button_disabled_value = /*$enforcementStore*/ ctx[0].isExecuting;
				attr_dev(button, "class", "inline-flex items-center px-6 py-3 border border-transparent text-base font-medium rounded-md shadow-sm text-white bg-red-600 hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 disabled:opacity-50 disabled:cursor-not-allowed");
				add_location(button, file$a, 172, 10, 7554);
				attr_dev(div0, "class", "mt-6");
				add_location(div0, file$a, 171, 8, 7525);
				add_location(p1, file$a, 193, 10, 8961);
				add_location(p2, file$a, 194, 10, 9024);
				attr_dev(div1, "class", "mt-4 text-xs text-gray-500");
				add_location(div1, file$a, 192, 8, 8910);
				attr_dev(div2, "class", "text-center");
				add_location(div2, file$a, 162, 6, 6946);
				attr_dev(div3, "class", "bg-white shadow rounded-lg p-6");
				add_location(div3, file$a, 161, 4, 6895);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div3, anchor);
				append_dev(div3, div2);
				append_dev(div2, svg);
				append_dev(svg, path);
				append_dev(div2, t0);
				append_dev(div2, h3);
				append_dev(div2, t2);
				append_dev(div2, p0);
				append_dev(div2, t4);
				append_dev(div2, div0);
				append_dev(div0, button);
				if_block.m(button, null);
				append_dev(div2, t5);
				append_dev(div2, div1);
				append_dev(div1, p1);
				append_dev(div1, t7);
				append_dev(div1, p2);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*executePlan*/ ctx[4], false, false, false, false);
					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (current_block_type !== (current_block_type = select_block_type_1(ctx))) {
					if_block.d(1);
					if_block = current_block_type(ctx);

					if (if_block) {
						if_block.c();
						if_block.m(button, null);
					}
				}

				if (dirty & /*$enforcementStore*/ 1 && button_disabled_value !== (button_disabled_value = /*$enforcementStore*/ ctx[0].isExecuting)) {
					prop_dev(button, "disabled", button_disabled_value);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div3);
				}

				if_block.d();
				mounted = false;
				dispose();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block$a.name,
			type: "else",
			source: "(150:2) {:else}",
			ctx
		});

		return block;
	}

	// (53:25) 
	function create_if_block_2$7(ctx) {
		let div15;
		let div1;
		let div0;
		let h3;
		let t1;
		let p;
		let t2;
		let span0;
		let t3_value = /*currentBatch*/ ctx[3].id.slice(0, 8) + "";
		let t3;
		let t4;
		let t5;
		let span1;
		let t6_value = /*currentBatch*/ ctx[3].status + "";
		let t6;
		let span1_class_value;
		let t7;
		let t8;
		let div14;
		let div4;
		let div2;
		let t9_value = /*currentBatch*/ ctx[3].summary.totalItems + "";
		let t9;
		let t10;
		let div3;
		let t12;
		let div7;
		let div5;
		let t13_value = /*currentBatch*/ ctx[3].summary.completedItems + "";
		let t13;
		let t14;
		let div6;
		let t16;
		let div10;
		let div8;
		let t17_value = /*currentBatch*/ ctx[3].summary.failedItems + "";
		let t17;
		let t18;
		let div9;
		let t20;
		let div13;
		let div11;
		let t21_value = /*currentBatch*/ ctx[3].summary.skippedItems + "";
		let t21;
		let t22;
		let div12;
		let t24;
		let if_block0 = /*progress*/ ctx[2] && create_if_block_5$6(ctx);
		let if_block1 = /*currentBatch*/ ctx[3].items.length > 0 && create_if_block_3$6(ctx);

		const block = {
			c: function create() {
				div15 = element("div");
				div1 = element("div");
				div0 = element("div");
				h3 = element("h3");
				h3.textContent = "Enforcement Execution";
				t1 = space();
				p = element("p");
				t2 = text("Batch ID: ");
				span0 = element("span");
				t3 = text(t3_value);
				t4 = text("...");
				t5 = space();
				span1 = element("span");
				t6 = text(t6_value);
				t7 = space();
				if (if_block0) if_block0.c();
				t8 = space();
				div14 = element("div");
				div4 = element("div");
				div2 = element("div");
				t9 = text(t9_value);
				t10 = space();
				div3 = element("div");
				div3.textContent = "Total Items";
				t12 = space();
				div7 = element("div");
				div5 = element("div");
				t13 = text(t13_value);
				t14 = space();
				div6 = element("div");
				div6.textContent = "Completed";
				t16 = space();
				div10 = element("div");
				div8 = element("div");
				t17 = text(t17_value);
				t18 = space();
				div9 = element("div");
				div9.textContent = "Failed";
				t20 = space();
				div13 = element("div");
				div11 = element("div");
				t21 = text(t21_value);
				t22 = space();
				div12 = element("div");
				div12.textContent = "Skipped";
				t24 = space();
				if (if_block1) if_block1.c();
				attr_dev(h3, "class", "text-lg font-medium text-gray-900");
				add_location(h3, file$a, 67, 10, 2557);
				attr_dev(span0, "class", "font-mono");
				add_location(span0, file$a, 69, 22, 2696);
				attr_dev(p, "class", "text-sm text-gray-500");
				add_location(p, file$a, 68, 10, 2640);
				add_location(div0, file$a, 66, 8, 2541);
				attr_dev(span1, "class", span1_class_value = "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium " + getStatusColor$1(/*currentBatch*/ ctx[3].status));
				add_location(span1, file$a, 72, 8, 2798);
				attr_dev(div1, "class", "flex items-center justify-between mb-4");
				add_location(div1, file$a, 65, 6, 2480);
				attr_dev(div2, "class", "text-lg font-semibold text-gray-900");
				add_location(div2, file$a, 101, 10, 3959);
				attr_dev(div3, "class", "text-xs text-gray-500");
				add_location(div3, file$a, 102, 10, 4058);
				attr_dev(div4, "class", "bg-gray-50 rounded-lg p-3 text-center");
				add_location(div4, file$a, 100, 8, 3897);
				attr_dev(div5, "class", "text-lg font-semibold text-green-600");
				add_location(div5, file$a, 105, 10, 4197);
				attr_dev(div6, "class", "text-xs text-gray-500");
				add_location(div6, file$a, 106, 10, 4301);
				attr_dev(div7, "class", "bg-green-50 rounded-lg p-3 text-center");
				add_location(div7, file$a, 104, 8, 4134);
				attr_dev(div8, "class", "text-lg font-semibold text-red-600");
				add_location(div8, file$a, 109, 10, 4436);
				attr_dev(div9, "class", "text-xs text-gray-500");
				add_location(div9, file$a, 110, 10, 4535);
				attr_dev(div10, "class", "bg-red-50 rounded-lg p-3 text-center");
				add_location(div10, file$a, 108, 8, 4375);
				attr_dev(div11, "class", "text-lg font-semibold text-yellow-600");
				add_location(div11, file$a, 113, 10, 4670);
				attr_dev(div12, "class", "text-xs text-gray-500");
				add_location(div12, file$a, 114, 10, 4773);
				attr_dev(div13, "class", "bg-yellow-50 rounded-lg p-3 text-center");
				add_location(div13, file$a, 112, 8, 4606);
				attr_dev(div14, "class", "grid grid-cols-1 gap-4 sm:grid-cols-4 mb-6");
				add_location(div14, file$a, 99, 6, 3832);
				attr_dev(div15, "class", "bg-white shadow rounded-lg p-6");
				add_location(div15, file$a, 64, 4, 2429);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div15, anchor);
				append_dev(div15, div1);
				append_dev(div1, div0);
				append_dev(div0, h3);
				append_dev(div0, t1);
				append_dev(div0, p);
				append_dev(p, t2);
				append_dev(p, span0);
				append_dev(span0, t3);
				append_dev(span0, t4);
				append_dev(div1, t5);
				append_dev(div1, span1);
				append_dev(span1, t6);
				append_dev(div15, t7);
				if (if_block0) if_block0.m(div15, null);
				append_dev(div15, t8);
				append_dev(div15, div14);
				append_dev(div14, div4);
				append_dev(div4, div2);
				append_dev(div2, t9);
				append_dev(div4, t10);
				append_dev(div4, div3);
				append_dev(div14, t12);
				append_dev(div14, div7);
				append_dev(div7, div5);
				append_dev(div5, t13);
				append_dev(div7, t14);
				append_dev(div7, div6);
				append_dev(div14, t16);
				append_dev(div14, div10);
				append_dev(div10, div8);
				append_dev(div8, t17);
				append_dev(div10, t18);
				append_dev(div10, div9);
				append_dev(div14, t20);
				append_dev(div14, div13);
				append_dev(div13, div11);
				append_dev(div11, t21);
				append_dev(div13, t22);
				append_dev(div13, div12);
				append_dev(div15, t24);
				if (if_block1) if_block1.m(div15, null);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*currentBatch*/ 8 && t3_value !== (t3_value = /*currentBatch*/ ctx[3].id.slice(0, 8) + "")) set_data_dev(t3, t3_value);
				if (dirty & /*currentBatch*/ 8 && t6_value !== (t6_value = /*currentBatch*/ ctx[3].status + "")) set_data_dev(t6, t6_value);

				if (dirty & /*currentBatch*/ 8 && span1_class_value !== (span1_class_value = "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium " + getStatusColor$1(/*currentBatch*/ ctx[3].status))) {
					attr_dev(span1, "class", span1_class_value);
				}

				if (/*progress*/ ctx[2]) {
					if (if_block0) {
						if_block0.p(ctx, dirty);
					} else {
						if_block0 = create_if_block_5$6(ctx);
						if_block0.c();
						if_block0.m(div15, t8);
					}
				} else if (if_block0) {
					if_block0.d(1);
					if_block0 = null;
				}

				if (dirty & /*currentBatch*/ 8 && t9_value !== (t9_value = /*currentBatch*/ ctx[3].summary.totalItems + "")) set_data_dev(t9, t9_value);
				if (dirty & /*currentBatch*/ 8 && t13_value !== (t13_value = /*currentBatch*/ ctx[3].summary.completedItems + "")) set_data_dev(t13, t13_value);
				if (dirty & /*currentBatch*/ 8 && t17_value !== (t17_value = /*currentBatch*/ ctx[3].summary.failedItems + "")) set_data_dev(t17, t17_value);
				if (dirty & /*currentBatch*/ 8 && t21_value !== (t21_value = /*currentBatch*/ ctx[3].summary.skippedItems + "")) set_data_dev(t21, t21_value);

				if (/*currentBatch*/ ctx[3].items.length > 0) {
					if (if_block1) {
						if_block1.p(ctx, dirty);
					} else {
						if_block1 = create_if_block_3$6(ctx);
						if_block1.c();
						if_block1.m(div15, null);
					}
				} else if (if_block1) {
					if_block1.d(1);
					if_block1 = null;
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div15);
				}

				if (if_block0) if_block0.d();
				if (if_block1) if_block1.d();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_2$7.name,
			type: "if",
			source: "(53:25) ",
			ctx
		});

		return block;
	}

	// (44:2) {#if !plan}
	function create_if_block_1$a(ctx) {
		let div;
		let svg;
		let path;
		let t0;
		let h3;
		let t2;
		let p;

		const block = {
			c: function create() {
				div = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				h3 = element("h3");
				h3.textContent = "No enforcement plan available";
				t2 = space();
				p = element("p");
				p.textContent = "Create a plan first to execute enforcement.";
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2");
				add_location(path, file$a, 57, 8, 1952);
				attr_dev(svg, "class", "mx-auto h-12 w-12 text-gray-400");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$a, 56, 6, 1844);
				attr_dev(h3, "class", "mt-2 text-sm font-medium text-gray-900");
				add_location(h3, file$a, 59, 6, 2175);
				attr_dev(p, "class", "mt-1 text-sm text-gray-500");
				add_location(p, file$a, 60, 6, 2267);
				attr_dev(div, "class", "text-center py-12");
				add_location(div, file$a, 55, 4, 1806);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, svg);
				append_dev(svg, path);
				append_dev(div, t0);
				append_dev(div, h3);
				append_dev(div, t2);
				append_dev(div, p);
			},
			p: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_1$a.name,
			type: "if",
			source: "(44:2) {#if !plan}",
			ctx
		});

		return block;
	}

	// (174:12) {:else}
	function create_else_block_1$5(ctx) {
		let svg;
		let path;
		let t;

		const block = {
			c: function create() {
				svg = svg_element("svg");
				path = svg_element("path");
				t = text("\n              Execute Enforcement Plan");
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M14.828 14.828a4 4 0 01-5.656 0M9 10h1.01M15 10h1.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z");
				add_location(path, file$a, 185, 16, 8624);
				attr_dev(svg, "class", "-ml-1 mr-3 h-5 w-5");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$a, 184, 14, 8521);
			},
			m: function mount(target, anchor) {
				insert_dev(target, svg, anchor);
				append_dev(svg, path);
				insert_dev(target, t, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(svg);
					detach_dev(t);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block_1$5.name,
			type: "else",
			source: "(174:12) {:else}",
			ctx
		});

		return block;
	}

	// (168:12) {#if $enforcementStore.isExecuting}
	function create_if_block_6$6(ctx) {
		let svg;
		let circle;
		let path;
		let t;

		const block = {
			c: function create() {
				svg = svg_element("svg");
				circle = svg_element("circle");
				path = svg_element("path");
				t = text("\n              Starting Execution...");
				attr_dev(circle, "class", "opacity-25");
				attr_dev(circle, "cx", "12");
				attr_dev(circle, "cy", "12");
				attr_dev(circle, "r", "10");
				attr_dev(circle, "stroke", "currentColor");
				attr_dev(circle, "stroke-width", "4");
				add_location(circle, file$a, 179, 16, 8146);
				attr_dev(path, "class", "opacity-75");
				attr_dev(path, "fill", "currentColor");
				attr_dev(path, "d", "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z");
				add_location(path, file$a, 180, 16, 8261);
				attr_dev(svg, "class", "animate-spin -ml-1 mr-3 h-5 w-5 text-white");
				attr_dev(svg, "xmlns", "http://www.w3.org/2000/svg");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				add_location(svg, file$a, 178, 14, 8006);
			},
			m: function mount(target, anchor) {
				insert_dev(target, svg, anchor);
				append_dev(svg, circle);
				append_dev(svg, path);
				insert_dev(target, t, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(svg);
					detach_dev(t);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_6$6.name,
			type: "if",
			source: "(168:12) {#if $enforcementStore.isExecuting}",
			ctx
		});

		return block;
	}

	// (69:6) {#if progress}
	function create_if_block_5$6(ctx) {
		let div4;
		let div0;
		let span0;
		let t1;
		let span1;
		let t2_value = /*progress*/ ctx[2].processed + "";
		let t2;
		let t3;
		let t4_value = /*progress*/ ctx[2].total + "";
		let t4;
		let t5;
		let t6_value = /*progress*/ ctx[2].percentage + "";
		let t6;
		let t7;
		let t8;
		let div2;
		let div1;
		let t9;
		let div3;
		let span2;
		let t10_value = /*progress*/ ctx[2].completed + "";
		let t10;
		let t11;
		let t12;
		let span3;
		let t13_value = /*progress*/ ctx[2].failed + "";
		let t13;
		let t14;
		let t15;
		let span4;
		let t16_value = /*progress*/ ctx[2].skipped + "";
		let t16;
		let t17;

		const block = {
			c: function create() {
				div4 = element("div");
				div0 = element("div");
				span0 = element("span");
				span0.textContent = "Progress";
				t1 = space();
				span1 = element("span");
				t2 = text(t2_value);
				t3 = text(" / ");
				t4 = text(t4_value);
				t5 = text(" (");
				t6 = text(t6_value);
				t7 = text("%)");
				t8 = space();
				div2 = element("div");
				div1 = element("div");
				t9 = space();
				div3 = element("div");
				span2 = element("span");
				t10 = text(t10_value);
				t11 = text(" completed");
				t12 = space();
				span3 = element("span");
				t13 = text(t13_value);
				t14 = text(" failed");
				t15 = space();
				span4 = element("span");
				t16 = text(t16_value);
				t17 = text(" skipped");
				add_location(span0, file$a, 81, 12, 3145);
				add_location(span1, file$a, 82, 12, 3179);
				attr_dev(div0, "class", "flex justify-between text-sm text-gray-600 mb-2");
				add_location(div0, file$a, 80, 10, 3071);
				attr_dev(div1, "class", "bg-indigo-600 h-2 rounded-full transition-all duration-300");
				set_style(div1, "width", /*progress*/ ctx[2].percentage + "%");
				add_location(div1, file$a, 85, 12, 3346);
				attr_dev(div2, "class", "w-full bg-gray-200 rounded-full h-2");
				add_location(div2, file$a, 84, 10, 3284);
				add_location(span2, file$a, 91, 12, 3606);
				add_location(span3, file$a, 92, 12, 3662);
				add_location(span4, file$a, 93, 12, 3712);
				attr_dev(div3, "class", "flex justify-between text-xs text-gray-500 mt-1");
				add_location(div3, file$a, 90, 10, 3532);
				attr_dev(div4, "class", "mb-6");
				add_location(div4, file$a, 79, 8, 3042);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div4, anchor);
				append_dev(div4, div0);
				append_dev(div0, span0);
				append_dev(div0, t1);
				append_dev(div0, span1);
				append_dev(span1, t2);
				append_dev(span1, t3);
				append_dev(span1, t4);
				append_dev(span1, t5);
				append_dev(span1, t6);
				append_dev(span1, t7);
				append_dev(div4, t8);
				append_dev(div4, div2);
				append_dev(div2, div1);
				append_dev(div4, t9);
				append_dev(div4, div3);
				append_dev(div3, span2);
				append_dev(span2, t10);
				append_dev(span2, t11);
				append_dev(div3, t12);
				append_dev(div3, span3);
				append_dev(span3, t13);
				append_dev(span3, t14);
				append_dev(div3, t15);
				append_dev(div3, span4);
				append_dev(span4, t16);
				append_dev(span4, t17);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*progress*/ 4 && t2_value !== (t2_value = /*progress*/ ctx[2].processed + "")) set_data_dev(t2, t2_value);
				if (dirty & /*progress*/ 4 && t4_value !== (t4_value = /*progress*/ ctx[2].total + "")) set_data_dev(t4, t4_value);
				if (dirty & /*progress*/ 4 && t6_value !== (t6_value = /*progress*/ ctx[2].percentage + "")) set_data_dev(t6, t6_value);

				if (dirty & /*progress*/ 4) {
					set_style(div1, "width", /*progress*/ ctx[2].percentage + "%");
				}

				if (dirty & /*progress*/ 4 && t10_value !== (t10_value = /*progress*/ ctx[2].completed + "")) set_data_dev(t10, t10_value);
				if (dirty & /*progress*/ 4 && t13_value !== (t13_value = /*progress*/ ctx[2].failed + "")) set_data_dev(t13, t13_value);
				if (dirty & /*progress*/ 4 && t16_value !== (t16_value = /*progress*/ ctx[2].skipped + "")) set_data_dev(t16, t16_value);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div4);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_5$6.name,
			type: "if",
			source: "(69:6) {#if progress}",
			ctx
		});

		return block;
	}

	// (110:6) {#if currentBatch.items.length > 0}
	function create_if_block_3$6(ctx) {
		let div1;
		let h4;
		let t1;
		let div0;
		let each_value = ensure_array_like_dev(/*currentBatch*/ ctx[3].items.slice(0, 10));
		let each_blocks = [];

		for (let i = 0; i < each_value.length; i += 1) {
			each_blocks[i] = create_each_block$4(get_each_context$4(ctx, each_value, i));
		}

		const block = {
			c: function create() {
				div1 = element("div");
				h4 = element("h4");
				h4.textContent = "Recent Actions";
				t1 = space();
				div0 = element("div");

				for (let i = 0; i < each_blocks.length; i += 1) {
					each_blocks[i].c();
				}

				attr_dev(h4, "class", "text-sm font-medium text-gray-900 mb-3");
				add_location(h4, file$a, 121, 10, 4947);
				attr_dev(div0, "class", "space-y-2 max-h-64 overflow-y-auto");
				add_location(div0, file$a, 122, 10, 5028);
				add_location(div1, file$a, 120, 8, 4931);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div1, anchor);
				append_dev(div1, h4);
				append_dev(div1, t1);
				append_dev(div1, div0);

				for (let i = 0; i < each_blocks.length; i += 1) {
					if (each_blocks[i]) {
						each_blocks[i].m(div0, null);
					}
				}
			},
			p: function update(ctx, dirty) {
				if (dirty & /*currentBatch, getStatusColor, getActionIcon*/ 8) {
					each_value = ensure_array_like_dev(/*currentBatch*/ ctx[3].items.slice(0, 10));
					let i;

					for (i = 0; i < each_value.length; i += 1) {
						const child_ctx = get_each_context$4(ctx, each_value, i);

						if (each_blocks[i]) {
							each_blocks[i].p(child_ctx, dirty);
						} else {
							each_blocks[i] = create_each_block$4(child_ctx);
							each_blocks[i].c();
							each_blocks[i].m(div0, null);
						}
					}

					for (; i < each_blocks.length; i += 1) {
						each_blocks[i].d(1);
					}

					each_blocks.length = each_value.length;
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div1);
				}

				destroy_each(each_blocks, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_3$6.name,
			type: "if",
			source: "(110:6) {#if currentBatch.items.length > 0}",
			ctx
		});

		return block;
	}

	// (133:18) {#if item.status === 'failed' && item.errorMessage}
	function create_if_block_4$6(ctx) {
		let button;
		let svg;
		let path;
		let button_title_value;

		const block = {
			c: function create() {
				button = element("button");
				svg = svg_element("svg");
				path = svg_element("path");
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z");
				add_location(path, file$a, 148, 24, 6522);
				attr_dev(svg, "class", "h-4 w-4");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$a, 147, 22, 6422);
				attr_dev(button, "title", button_title_value = /*item*/ ctx[7].errorMessage);
				attr_dev(button, "class", "text-red-400 hover:text-red-600");
				add_location(button, file$a, 143, 20, 6260);
			},
			m: function mount(target, anchor) {
				insert_dev(target, button, anchor);
				append_dev(button, svg);
				append_dev(svg, path);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*currentBatch*/ 8 && button_title_value !== (button_title_value = /*item*/ ctx[7].errorMessage)) {
					attr_dev(button, "title", button_title_value);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(button);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_4$6.name,
			type: "if",
			source: "(133:18) {#if item.status === 'failed' && item.errorMessage}",
			ctx
		});

		return block;
	}

	// (114:12) {#each currentBatch.items.slice(0, 10) as item}
	function create_each_block$4(ctx) {
		let div5;
		let div3;
		let svg;
		let path;
		let path_d_value;
		let t0;
		let div2;
		let div0;
		let t1_value = /*item*/ ctx[7].action.replace(/_/g, ' ') + "";
		let t1;
		let t2;
		let div1;
		let t3_value = /*item*/ ctx[7].entityType + "";
		let t3;
		let t4;
		let t5_value = /*item*/ ctx[7].entityId.slice(0, 20) + "";
		let t5;
		let t6;
		let t7;
		let div4;
		let span;
		let t8_value = /*item*/ ctx[7].status + "";
		let t8;
		let span_class_value;
		let t9;
		let t10;
		let if_block = /*item*/ ctx[7].status === 'failed' && /*item*/ ctx[7].errorMessage && create_if_block_4$6(ctx);

		const block = {
			c: function create() {
				div5 = element("div");
				div3 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				div2 = element("div");
				div0 = element("div");
				t1 = text(t1_value);
				t2 = space();
				div1 = element("div");
				t3 = text(t3_value);
				t4 = text(": ");
				t5 = text(t5_value);
				t6 = text("...");
				t7 = space();
				div4 = element("div");
				span = element("span");
				t8 = text(t8_value);
				t9 = space();
				if (if_block) if_block.c();
				t10 = space();
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", path_d_value = getActionIcon(/*item*/ ctx[7].action));
				add_location(path, file$a, 127, 20, 5417);
				attr_dev(svg, "class", "h-4 w-4 text-gray-400");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$a, 126, 18, 5307);
				attr_dev(div0, "class", "text-sm font-medium text-gray-900");
				add_location(div0, file$a, 130, 20, 5590);
				attr_dev(div1, "class", "text-xs text-gray-500");
				add_location(div1, file$a, 133, 20, 5740);
				add_location(div2, file$a, 129, 18, 5564);
				attr_dev(div3, "class", "flex items-center space-x-3");
				add_location(div3, file$a, 125, 16, 5247);
				attr_dev(span, "class", span_class_value = "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium " + getStatusColor$1(/*item*/ ctx[7].status));
				add_location(span, file$a, 139, 18, 6000);
				attr_dev(div4, "class", "flex items-center space-x-2");
				add_location(div4, file$a, 138, 16, 5940);
				attr_dev(div5, "class", "flex items-center justify-between py-2 px-3 bg-gray-50 rounded-md");
				add_location(div5, file$a, 124, 14, 5151);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div5, anchor);
				append_dev(div5, div3);
				append_dev(div3, svg);
				append_dev(svg, path);
				append_dev(div3, t0);
				append_dev(div3, div2);
				append_dev(div2, div0);
				append_dev(div0, t1);
				append_dev(div2, t2);
				append_dev(div2, div1);
				append_dev(div1, t3);
				append_dev(div1, t4);
				append_dev(div1, t5);
				append_dev(div1, t6);
				append_dev(div5, t7);
				append_dev(div5, div4);
				append_dev(div4, span);
				append_dev(span, t8);
				append_dev(div4, t9);
				if (if_block) if_block.m(div4, null);
				append_dev(div5, t10);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*currentBatch*/ 8 && path_d_value !== (path_d_value = getActionIcon(/*item*/ ctx[7].action))) {
					attr_dev(path, "d", path_d_value);
				}

				if (dirty & /*currentBatch*/ 8 && t1_value !== (t1_value = /*item*/ ctx[7].action.replace(/_/g, ' ') + "")) set_data_dev(t1, t1_value);
				if (dirty & /*currentBatch*/ 8 && t3_value !== (t3_value = /*item*/ ctx[7].entityType + "")) set_data_dev(t3, t3_value);
				if (dirty & /*currentBatch*/ 8 && t5_value !== (t5_value = /*item*/ ctx[7].entityId.slice(0, 20) + "")) set_data_dev(t5, t5_value);
				if (dirty & /*currentBatch*/ 8 && t8_value !== (t8_value = /*item*/ ctx[7].status + "")) set_data_dev(t8, t8_value);

				if (dirty & /*currentBatch*/ 8 && span_class_value !== (span_class_value = "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium " + getStatusColor$1(/*item*/ ctx[7].status))) {
					attr_dev(span, "class", span_class_value);
				}

				if (/*item*/ ctx[7].status === 'failed' && /*item*/ ctx[7].errorMessage) {
					if (if_block) {
						if_block.p(ctx, dirty);
					} else {
						if_block = create_if_block_4$6(ctx);
						if_block.c();
						if_block.m(div4, null);
					}
				} else if (if_block) {
					if_block.d(1);
					if_block = null;
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div5);
				}

				if (if_block) if_block.d();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_each_block$4.name,
			type: "each",
			source: "(114:12) {#each currentBatch.items.slice(0, 10) as item}",
			ctx
		});

		return block;
	}

	// (192:2) {#if $enforcementStore.error}
	function create_if_block$a(ctx) {
		let div3;
		let div2;
		let div0;
		let svg;
		let path;
		let t0;
		let div1;
		let p;
		let t1_value = /*$enforcementStore*/ ctx[0].error + "";
		let t1;
		let t2;
		let button;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				div3 = element("div");
				div2 = element("div");
				div0 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				div1 = element("div");
				p = element("p");
				t1 = text(t1_value);
				t2 = space();
				button = element("button");
				button.textContent = "Dismiss";
				attr_dev(path, "fill-rule", "evenodd");
				attr_dev(path, "d", "M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z");
				attr_dev(path, "clip-rule", "evenodd");
				add_location(path, file$a, 206, 12, 9394);
				attr_dev(svg, "class", "h-5 w-5 text-red-400");
				attr_dev(svg, "viewBox", "0 0 20 20");
				attr_dev(svg, "fill", "currentColor");
				add_location(svg, file$a, 205, 10, 9307);
				attr_dev(div0, "class", "flex-shrink-0");
				add_location(div0, file$a, 204, 8, 9269);
				attr_dev(p, "class", "text-sm text-red-800");
				add_location(p, file$a, 210, 10, 9732);
				attr_dev(button, "class", "mt-2 text-sm text-red-600 hover:text-red-500");
				add_location(button, file$a, 211, 10, 9804);
				attr_dev(div1, "class", "ml-3");
				add_location(div1, file$a, 209, 8, 9703);
				attr_dev(div2, "class", "flex");
				add_location(div2, file$a, 203, 6, 9242);
				attr_dev(div3, "class", "bg-red-50 border border-red-200 rounded-md p-4");
				add_location(div3, file$a, 202, 4, 9175);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div3, anchor);
				append_dev(div3, div2);
				append_dev(div2, div0);
				append_dev(div0, svg);
				append_dev(svg, path);
				append_dev(div2, t0);
				append_dev(div2, div1);
				append_dev(div1, p);
				append_dev(p, t1);
				append_dev(div1, t2);
				append_dev(div1, button);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*click_handler*/ ctx[6], false, false, false, false);
					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (dirty & /*$enforcementStore*/ 1 && t1_value !== (t1_value = /*$enforcementStore*/ ctx[0].error + "")) set_data_dev(t1, t1_value);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div3);
				}

				mounted = false;
				dispose();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block$a.name,
			type: "if",
			source: "(192:2) {#if $enforcementStore.error}",
			ctx
		});

		return block;
	}

	function create_fragment$a(ctx) {
		let div;
		let t;

		function select_block_type(ctx, dirty) {
			if (!/*plan*/ ctx[1]) return create_if_block_1$a;
			if (/*currentBatch*/ ctx[3]) return create_if_block_2$7;
			return create_else_block$a;
		}

		let current_block_type = select_block_type(ctx);
		let if_block0 = current_block_type(ctx);
		let if_block1 = /*$enforcementStore*/ ctx[0].error && create_if_block$a(ctx);

		const block = {
			c: function create() {
				div = element("div");
				if_block0.c();
				t = space();
				if (if_block1) if_block1.c();
				attr_dev(div, "class", "space-y-6");
				add_location(div, file$a, 52, 0, 1733);
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				if_block0.m(div, null);
				append_dev(div, t);
				if (if_block1) if_block1.m(div, null);
			},
			p: function update(ctx, [dirty]) {
				if (current_block_type === (current_block_type = select_block_type(ctx)) && if_block0) {
					if_block0.p(ctx, dirty);
				} else {
					if_block0.d(1);
					if_block0 = current_block_type(ctx);

					if (if_block0) {
						if_block0.c();
						if_block0.m(div, t);
					}
				}

				if (/*$enforcementStore*/ ctx[0].error) {
					if (if_block1) {
						if_block1.p(ctx, dirty);
					} else {
						if_block1 = create_if_block$a(ctx);
						if_block1.c();
						if_block1.m(div, null);
					}
				} else if (if_block1) {
					if_block1.d(1);
					if_block1 = null;
				}
			},
			i: noop,
			o: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}

				if_block0.d();
				if (if_block1) if_block1.d();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_fragment$a.name,
			type: "component",
			source: "",
			ctx
		});

		return block;
	}

	function getStatusColor$1(status) {
		switch (status) {
			case 'pending':
				return 'text-gray-600 bg-gray-100';
			case 'running':
				return 'text-blue-600 bg-blue-100';
			case 'completed':
				return 'text-green-600 bg-green-100';
			case 'failed':
				return 'text-red-600 bg-red-100';
			case 'cancelled':
				return 'text-yellow-600 bg-yellow-100';
			default:
				return 'text-gray-600 bg-gray-100';
		}
	}

	function getActionIcon(action) {
		switch (action) {
			case 'remove_liked_song':
				return 'M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z';
			case 'remove_playlist_track':
				return 'M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3';
			case 'unfollow_artist':
				return 'M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z';
			default:
				return 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z';
		}
	}

	function instance$a($$self, $$props, $$invalidate) {
		let plan;
		let currentBatch;
		let progress;
		let $executionProgress;
		let $enforcementStore;
		validate_store(executionProgress, 'executionProgress');
		component_subscribe($$self, executionProgress, $$value => $$invalidate(5, $executionProgress = $$value));
		validate_store(enforcementStore, 'enforcementStore');
		component_subscribe($$self, enforcementStore, $$value => $$invalidate(0, $enforcementStore = $$value));
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('EnforcementExecution', slots, []);

		async function executePlan() {
			if (!plan) return;
			const confirmed = confirm('Are you sure you want to execute this enforcement plan? This will modify your music library and some changes may not be reversible.');

			if (confirmed) {
				await enforcementActions.executePlan(plan.planId);
			}
		}

		const writable_props = [];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<EnforcementExecution> was created with unknown prop '${key}'`);
		});

		const click_handler = () => enforcementActions.clearError();

		$$self.$capture_state = () => ({
			enforcementActions,
			enforcementStore,
			executionProgress,
			executePlan,
			getStatusColor: getStatusColor$1,
			getActionIcon,
			plan,
			progress,
			currentBatch,
			$executionProgress,
			$enforcementStore
		});

		$$self.$inject_state = $$props => {
			if ('plan' in $$props) $$invalidate(1, plan = $$props.plan);
			if ('progress' in $$props) $$invalidate(2, progress = $$props.progress);
			if ('currentBatch' in $$props) $$invalidate(3, currentBatch = $$props.currentBatch);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		$$self.$$.update = () => {
			if ($$self.$$.dirty & /*$enforcementStore*/ 1) {
				$$invalidate(1, plan = $enforcementStore.currentPlan);
			}

			if ($$self.$$.dirty & /*$enforcementStore*/ 1) {
				$$invalidate(3, currentBatch = $enforcementStore.currentBatch);
			}

			if ($$self.$$.dirty & /*$executionProgress*/ 32) {
				$$invalidate(2, progress = $executionProgress);
			}
		};

		return [
			$enforcementStore,
			plan,
			progress,
			currentBatch,
			executePlan,
			$executionProgress,
			click_handler
		];
	}

	class EnforcementExecution extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance$a, create_fragment$a, safe_not_equal, {});

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "EnforcementExecution",
				options,
				id: create_fragment$a.name
			});
		}
	}

	/* src/lib/components/ActionHistory.svelte generated by Svelte v4.2.20 */

	const file$9 = "src/lib/components/ActionHistory.svelte";

	function get_each_context$3(ctx, list, i) {
		const child_ctx = ctx.slice();
		child_ctx[5] = list[i];
		return child_ctx;
	}

	// (47:4) {#if $canRollback}
	function create_if_block_6$5(ctx) {
		let div;
		let svg;
		let path;
		let t;

		const block = {
			c: function create() {
				div = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t = text("\n        Rollback available");
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z");
				add_location(path, file$9, 58, 10, 1800);
				attr_dev(svg, "class", "inline h-4 w-4 text-green-400");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$9, 57, 8, 1692);
				attr_dev(div, "class", "text-sm text-gray-500");
				add_location(div, file$9, 56, 6, 1648);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, svg);
				append_dev(svg, path);
				append_dev(div, t);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_6$5.name,
			type: "if",
			source: "(47:4) {#if $canRollback}",
			ctx
		});

		return block;
	}

	// (68:2) {:else}
	function create_else_block$9(ctx) {
		let div;
		let ul;
		let each_value = ensure_array_like_dev(/*actionHistory*/ ctx[0]);
		let each_blocks = [];

		for (let i = 0; i < each_value.length; i += 1) {
			each_blocks[i] = create_each_block$3(get_each_context$3(ctx, each_value, i));
		}

		const block = {
			c: function create() {
				div = element("div");
				ul = element("ul");

				for (let i = 0; i < each_blocks.length; i += 1) {
					each_blocks[i].c();
				}

				attr_dev(ul, "class", "divide-y divide-gray-200");
				add_location(ul, file$9, 79, 6, 2673);
				attr_dev(div, "class", "bg-white shadow overflow-hidden sm:rounded-md");
				add_location(div, file$9, 78, 4, 2607);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, ul);

				for (let i = 0; i < each_blocks.length; i += 1) {
					if (each_blocks[i]) {
						each_blocks[i].m(ul, null);
					}
				}
			},
			p: function update(ctx, dirty) {
				if (dirty & /*actionHistory, rollbackBatch, getSuccessRate, formatDate, getStatusColor*/ 5) {
					each_value = ensure_array_like_dev(/*actionHistory*/ ctx[0]);
					let i;

					for (i = 0; i < each_value.length; i += 1) {
						const child_ctx = get_each_context$3(ctx, each_value, i);

						if (each_blocks[i]) {
							each_blocks[i].p(child_ctx, dirty);
						} else {
							each_blocks[i] = create_each_block$3(child_ctx);
							each_blocks[i].c();
							each_blocks[i].m(ul, null);
						}
					}

					for (; i < each_blocks.length; i += 1) {
						each_blocks[i].d(1);
					}

					each_blocks.length = each_value.length;
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
			id: create_else_block$9.name,
			type: "else",
			source: "(68:2) {:else}",
			ctx
		});

		return block;
	}

	// (57:2) {#if actionHistory.length === 0}
	function create_if_block$9(ctx) {
		let div;
		let svg;
		let path;
		let t0;
		let h3;
		let t2;
		let p;

		const block = {
			c: function create() {
				div = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				h3 = element("h3");
				h3.textContent = "No enforcement history";
				t2 = space();
				p = element("p");
				p.textContent = "Your enforcement executions will appear here after you run them.";
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z");
				add_location(path, file$9, 69, 8, 2208);
				attr_dev(svg, "class", "mx-auto h-12 w-12 text-gray-400");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$9, 68, 6, 2100);
				attr_dev(h3, "class", "mt-2 text-sm font-medium text-gray-900");
				add_location(h3, file$9, 71, 6, 2348);
				attr_dev(p, "class", "mt-1 text-sm text-gray-500");
				add_location(p, file$9, 72, 6, 2433);
				attr_dev(div, "class", "text-center py-12");
				add_location(div, file$9, 67, 4, 2062);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, svg);
				append_dev(svg, path);
				append_dev(div, t0);
				append_dev(div, h3);
				append_dev(div, t2);
				append_dev(div, p);
			},
			p: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block$9.name,
			type: "if",
			source: "(57:2) {#if actionHistory.length === 0}",
			ctx
		});

		return block;
	}

	// (94:24) {#if batch.completedAt}
	function create_if_block_5$5(ctx) {
		let t0;
		let t1_value = formatDate$4(/*batch*/ ctx[5].completedAt) + "";
		let t1;

		const block = {
			c: function create() {
				t0 = text("• Completed ");
				t1 = text(t1_value);
			},
			m: function mount(target, anchor) {
				insert_dev(target, t0, anchor);
				insert_dev(target, t1, anchor);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*actionHistory*/ 1 && t1_value !== (t1_value = formatDate$4(/*batch*/ ctx[5].completedAt) + "")) set_data_dev(t1, t1_value);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(t0);
					detach_dev(t1);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_5$5.name,
			type: "if",
			source: "(94:24) {#if batch.completedAt}",
			ctx
		});

		return block;
	}

	// (117:18) {#if batch.status === 'completed' && batch.summary.completedItems > 0}
	function create_if_block_4$5(ctx) {
		let button;
		let svg;
		let path;
		let t;
		let mounted;
		let dispose;

		function click_handler() {
			return /*click_handler*/ ctx[4](/*batch*/ ctx[5]);
		}

		const block = {
			c: function create() {
				button = element("button");
				svg = svg_element("svg");
				path = svg_element("path");
				t = text("\n                      Rollback");
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M3 10h10a8 8 0 018 8v2M3 10l6 6m-6-6l6-6");
				add_location(path, file$9, 131, 24, 5959);
				attr_dev(svg, "class", "-ml-0.5 mr-2 h-4 w-4");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$9, 130, 22, 5846);
				attr_dev(button, "class", "inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500");
				add_location(button, file$9, 126, 20, 5478);
			},
			m: function mount(target, anchor) {
				insert_dev(target, button, anchor);
				append_dev(button, svg);
				append_dev(svg, path);
				append_dev(button, t);

				if (!mounted) {
					dispose = listen_dev(button, "click", click_handler, false, false, false, false);
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
			id: create_if_block_4$5.name,
			type: "if",
			source: "(117:18) {#if batch.status === 'completed' && batch.summary.completedItems > 0}",
			ctx
		});

		return block;
	}

	// (158:18) {#if batch.options.blockCollabs}
	function create_if_block_3$5(ctx) {
		let t;

		const block = {
			c: function create() {
				t = text(", block collaborations");
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
			id: create_if_block_3$5.name,
			type: "if",
			source: "(158:18) {#if batch.options.blockCollabs}",
			ctx
		});

		return block;
	}

	// (159:18) {#if batch.options.blockFeaturing}
	function create_if_block_2$6(ctx) {
		let t;

		const block = {
			c: function create() {
				t = text(", block featuring");
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
			id: create_if_block_2$6.name,
			type: "if",
			source: "(159:18) {#if batch.options.blockFeaturing}",
			ctx
		});

		return block;
	}

	// (160:18) {#if batch.options.blockSongwriterOnly}
	function create_if_block_1$9(ctx) {
		let t;

		const block = {
			c: function create() {
				t = text(", block songwriter credits");
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
			id: create_if_block_1$9.name,
			type: "if",
			source: "(160:18) {#if batch.options.blockSongwriterOnly}",
			ctx
		});

		return block;
	}

	// (72:8) {#each actionHistory as batch}
	function create_each_block$3(ctx) {
		let li;
		let div27;
		let div10;
		let div5;
		let div0;
		let svg;
		let path;
		let t0;
		let div4;
		let div1;
		let p0;
		let t1_value = /*batch*/ ctx[5].provider + "";
		let t1;
		let t2;
		let t3;
		let span0;
		let t4_value = /*batch*/ ctx[5].status + "";
		let t4;
		let span0_class_value;
		let t5;
		let div2;
		let p1;
		let t6;
		let t7_value = formatDate$4(/*batch*/ ctx[5].createdAt) + "";
		let t7;
		let t8;
		let t9;
		let div3;
		let t10;
		let span1;
		let t11_value = /*batch*/ ctx[5].id.slice(0, 8) + "";
		let t11;
		let t12;
		let t13;
		let div9;
		let div8;
		let div6;
		let t14_value = getSuccessRate(/*batch*/ ctx[5]) + "";
		let t14;
		let t15;
		let t16;
		let div7;
		let t17_value = /*batch*/ ctx[5].summary.completedItems + "";
		let t17;
		let t18;
		let t19_value = /*batch*/ ctx[5].summary.totalItems + "";
		let t19;
		let t20;
		let t21;
		let t22;
		let div24;
		let div23;
		let div13;
		let div11;
		let t23_value = /*batch*/ ctx[5].summary.totalItems + "";
		let t23;
		let t24;
		let div12;
		let t26;
		let div16;
		let div14;
		let t27_value = /*batch*/ ctx[5].summary.completedItems + "";
		let t27;
		let t28;
		let div15;
		let t30;
		let div19;
		let div17;
		let t31_value = /*batch*/ ctx[5].summary.failedItems + "";
		let t31;
		let t32;
		let div18;
		let t34;
		let div22;
		let div20;
		let t35_value = /*batch*/ ctx[5].summary.skippedItems + "";
		let t35;
		let t36;
		let div21;
		let t38;
		let div26;
		let div25;
		let span2;
		let t40;
		let t41_value = /*batch*/ ctx[5].options.aggressiveness + "";
		let t41;
		let t42;
		let t43;
		let t44;
		let t45;
		let if_block0 = /*batch*/ ctx[5].completedAt && create_if_block_5$5(ctx);
		let if_block1 = /*batch*/ ctx[5].status === 'completed' && /*batch*/ ctx[5].summary.completedItems > 0 && create_if_block_4$5(ctx);
		let if_block2 = /*batch*/ ctx[5].options.blockCollabs && create_if_block_3$5(ctx);
		let if_block3 = /*batch*/ ctx[5].options.blockFeaturing && create_if_block_2$6(ctx);
		let if_block4 = /*batch*/ ctx[5].options.blockSongwriterOnly && create_if_block_1$9(ctx);

		const block = {
			c: function create() {
				li = element("li");
				div27 = element("div");
				div10 = element("div");
				div5 = element("div");
				div0 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				div4 = element("div");
				div1 = element("div");
				p0 = element("p");
				t1 = text(t1_value);
				t2 = text(" Enforcement");
				t3 = space();
				span0 = element("span");
				t4 = text(t4_value);
				t5 = space();
				div2 = element("div");
				p1 = element("p");
				t6 = text("Executed ");
				t7 = text(t7_value);
				t8 = space();
				if (if_block0) if_block0.c();
				t9 = space();
				div3 = element("div");
				t10 = text("Batch ID: ");
				span1 = element("span");
				t11 = text(t11_value);
				t12 = text("...");
				t13 = space();
				div9 = element("div");
				div8 = element("div");
				div6 = element("div");
				t14 = text(t14_value);
				t15 = text("% success");
				t16 = space();
				div7 = element("div");
				t17 = text(t17_value);
				t18 = text(" / ");
				t19 = text(t19_value);
				t20 = text(" items");
				t21 = space();
				if (if_block1) if_block1.c();
				t22 = space();
				div24 = element("div");
				div23 = element("div");
				div13 = element("div");
				div11 = element("div");
				t23 = text(t23_value);
				t24 = space();
				div12 = element("div");
				div12.textContent = "Total";
				t26 = space();
				div16 = element("div");
				div14 = element("div");
				t27 = text(t27_value);
				t28 = space();
				div15 = element("div");
				div15.textContent = "Completed";
				t30 = space();
				div19 = element("div");
				div17 = element("div");
				t31 = text(t31_value);
				t32 = space();
				div18 = element("div");
				div18.textContent = "Failed";
				t34 = space();
				div22 = element("div");
				div20 = element("div");
				t35 = text(t35_value);
				t36 = space();
				div21 = element("div");
				div21.textContent = "Skipped";
				t38 = space();
				div26 = element("div");
				div25 = element("div");
				span2 = element("span");
				span2.textContent = "Options:";
				t40 = space();
				t41 = text(t41_value);
				t42 = text(" aggressiveness\n                  ");
				if (if_block2) if_block2.c();
				t43 = space();
				if (if_block3) if_block3.c();
				t44 = space();
				if (if_block4) if_block4.c();
				t45 = space();
				attr_dev(path, "d", "M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.42 1.56-.299.421-1.02.599-1.559.3z");
				add_location(path, file$9, 87, 22, 3084);
				attr_dev(svg, "class", "h-8 w-8 text-green-500");
				attr_dev(svg, "fill", "currentColor");
				attr_dev(svg, "viewBox", "0 0 24 24");
				add_location(svg, file$9, 86, 20, 2985);
				attr_dev(div0, "class", "flex-shrink-0");
				add_location(div0, file$9, 85, 18, 2937);
				attr_dev(p0, "class", "text-sm font-medium text-gray-900 capitalize");
				add_location(p0, file$9, 92, 22, 3845);
				attr_dev(span0, "class", span0_class_value = "ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium " + getStatusColor(/*batch*/ ctx[5].status));
				add_location(span0, file$9, 95, 22, 4004);
				attr_dev(div1, "class", "flex items-center");
				add_location(div1, file$9, 91, 20, 3791);
				add_location(p1, file$9, 100, 22, 4324);
				attr_dev(div2, "class", "mt-1 flex items-center text-sm text-gray-500");
				add_location(div2, file$9, 99, 20, 4243);
				attr_dev(span1, "class", "font-mono");
				add_location(span1, file$9, 108, 32, 4686);
				attr_dev(div3, "class", "mt-1 text-xs text-gray-400");
				add_location(div3, file$9, 107, 20, 4613);
				attr_dev(div4, "class", "ml-4");
				add_location(div4, file$9, 90, 18, 3752);
				attr_dev(div5, "class", "flex items-center");
				add_location(div5, file$9, 84, 16, 2887);
				attr_dev(div6, "class", "text-sm font-medium text-gray-900");
				add_location(div6, file$9, 116, 20, 4989);
				attr_dev(div7, "class", "text-xs text-gray-500");
				add_location(div7, file$9, 119, 20, 5139);
				attr_dev(div8, "class", "text-right");
				add_location(div8, file$9, 115, 18, 4944);
				attr_dev(div9, "class", "flex items-center space-x-4");
				add_location(div9, file$9, 113, 16, 4851);
				attr_dev(div10, "class", "flex items-center justify-between");
				add_location(div10, file$9, 83, 14, 2823);
				attr_dev(div11, "class", "text-lg font-semibold text-gray-900");
				add_location(div11, file$9, 143, 20, 6453);
				attr_dev(div12, "class", "text-xs text-gray-500");
				add_location(div12, file$9, 144, 20, 6555);
				attr_dev(div13, "class", "text-center");
				add_location(div13, file$9, 142, 18, 6407);
				attr_dev(div14, "class", "text-lg font-semibold text-green-600");
				add_location(div14, file$9, 147, 20, 6691);
				attr_dev(div15, "class", "text-xs text-gray-500");
				add_location(div15, file$9, 148, 20, 6798);
				attr_dev(div16, "class", "text-center");
				add_location(div16, file$9, 146, 18, 6645);
				attr_dev(div17, "class", "text-lg font-semibold text-red-600");
				add_location(div17, file$9, 151, 20, 6938);
				attr_dev(div18, "class", "text-xs text-gray-500");
				add_location(div18, file$9, 152, 20, 7040);
				attr_dev(div19, "class", "text-center");
				add_location(div19, file$9, 150, 18, 6892);
				attr_dev(div20, "class", "text-lg font-semibold text-yellow-600");
				add_location(div20, file$9, 155, 20, 7177);
				attr_dev(div21, "class", "text-xs text-gray-500");
				add_location(div21, file$9, 156, 20, 7283);
				attr_dev(div22, "class", "text-center");
				add_location(div22, file$9, 154, 18, 7131);
				attr_dev(div23, "class", "grid grid-cols-2 gap-4 sm:grid-cols-4");
				add_location(div23, file$9, 141, 16, 6337);
				attr_dev(div24, "class", "mt-4");
				add_location(div24, file$9, 140, 14, 6302);
				attr_dev(span2, "class", "font-medium");
				add_location(span2, file$9, 164, 18, 7571);
				attr_dev(div25, "class", "text-xs text-gray-500");
				add_location(div25, file$9, 163, 16, 7517);
				attr_dev(div26, "class", "mt-3 pt-3 border-t border-gray-200");
				add_location(div26, file$9, 162, 14, 7452);
				attr_dev(div27, "class", "px-4 py-4 sm:px-6");
				add_location(div27, file$9, 82, 12, 2777);
				add_location(li, file$9, 81, 10, 2760);
			},
			m: function mount(target, anchor) {
				insert_dev(target, li, anchor);
				append_dev(li, div27);
				append_dev(div27, div10);
				append_dev(div10, div5);
				append_dev(div5, div0);
				append_dev(div0, svg);
				append_dev(svg, path);
				append_dev(div5, t0);
				append_dev(div5, div4);
				append_dev(div4, div1);
				append_dev(div1, p0);
				append_dev(p0, t1);
				append_dev(p0, t2);
				append_dev(div1, t3);
				append_dev(div1, span0);
				append_dev(span0, t4);
				append_dev(div4, t5);
				append_dev(div4, div2);
				append_dev(div2, p1);
				append_dev(p1, t6);
				append_dev(p1, t7);
				append_dev(p1, t8);
				if (if_block0) if_block0.m(p1, null);
				append_dev(div4, t9);
				append_dev(div4, div3);
				append_dev(div3, t10);
				append_dev(div3, span1);
				append_dev(span1, t11);
				append_dev(span1, t12);
				append_dev(div10, t13);
				append_dev(div10, div9);
				append_dev(div9, div8);
				append_dev(div8, div6);
				append_dev(div6, t14);
				append_dev(div6, t15);
				append_dev(div8, t16);
				append_dev(div8, div7);
				append_dev(div7, t17);
				append_dev(div7, t18);
				append_dev(div7, t19);
				append_dev(div7, t20);
				append_dev(div9, t21);
				if (if_block1) if_block1.m(div9, null);
				append_dev(div27, t22);
				append_dev(div27, div24);
				append_dev(div24, div23);
				append_dev(div23, div13);
				append_dev(div13, div11);
				append_dev(div11, t23);
				append_dev(div13, t24);
				append_dev(div13, div12);
				append_dev(div23, t26);
				append_dev(div23, div16);
				append_dev(div16, div14);
				append_dev(div14, t27);
				append_dev(div16, t28);
				append_dev(div16, div15);
				append_dev(div23, t30);
				append_dev(div23, div19);
				append_dev(div19, div17);
				append_dev(div17, t31);
				append_dev(div19, t32);
				append_dev(div19, div18);
				append_dev(div23, t34);
				append_dev(div23, div22);
				append_dev(div22, div20);
				append_dev(div20, t35);
				append_dev(div22, t36);
				append_dev(div22, div21);
				append_dev(div27, t38);
				append_dev(div27, div26);
				append_dev(div26, div25);
				append_dev(div25, span2);
				append_dev(div25, t40);
				append_dev(div25, t41);
				append_dev(div25, t42);
				if (if_block2) if_block2.m(div25, null);
				append_dev(div25, t43);
				if (if_block3) if_block3.m(div25, null);
				append_dev(div25, t44);
				if (if_block4) if_block4.m(div25, null);
				append_dev(li, t45);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*actionHistory*/ 1 && t1_value !== (t1_value = /*batch*/ ctx[5].provider + "")) set_data_dev(t1, t1_value);
				if (dirty & /*actionHistory*/ 1 && t4_value !== (t4_value = /*batch*/ ctx[5].status + "")) set_data_dev(t4, t4_value);

				if (dirty & /*actionHistory*/ 1 && span0_class_value !== (span0_class_value = "ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium " + getStatusColor(/*batch*/ ctx[5].status))) {
					attr_dev(span0, "class", span0_class_value);
				}

				if (dirty & /*actionHistory*/ 1 && t7_value !== (t7_value = formatDate$4(/*batch*/ ctx[5].createdAt) + "")) set_data_dev(t7, t7_value);

				if (/*batch*/ ctx[5].completedAt) {
					if (if_block0) {
						if_block0.p(ctx, dirty);
					} else {
						if_block0 = create_if_block_5$5(ctx);
						if_block0.c();
						if_block0.m(p1, null);
					}
				} else if (if_block0) {
					if_block0.d(1);
					if_block0 = null;
				}

				if (dirty & /*actionHistory*/ 1 && t11_value !== (t11_value = /*batch*/ ctx[5].id.slice(0, 8) + "")) set_data_dev(t11, t11_value);
				if (dirty & /*actionHistory*/ 1 && t14_value !== (t14_value = getSuccessRate(/*batch*/ ctx[5]) + "")) set_data_dev(t14, t14_value);
				if (dirty & /*actionHistory*/ 1 && t17_value !== (t17_value = /*batch*/ ctx[5].summary.completedItems + "")) set_data_dev(t17, t17_value);
				if (dirty & /*actionHistory*/ 1 && t19_value !== (t19_value = /*batch*/ ctx[5].summary.totalItems + "")) set_data_dev(t19, t19_value);

				if (/*batch*/ ctx[5].status === 'completed' && /*batch*/ ctx[5].summary.completedItems > 0) {
					if (if_block1) {
						if_block1.p(ctx, dirty);
					} else {
						if_block1 = create_if_block_4$5(ctx);
						if_block1.c();
						if_block1.m(div9, null);
					}
				} else if (if_block1) {
					if_block1.d(1);
					if_block1 = null;
				}

				if (dirty & /*actionHistory*/ 1 && t23_value !== (t23_value = /*batch*/ ctx[5].summary.totalItems + "")) set_data_dev(t23, t23_value);
				if (dirty & /*actionHistory*/ 1 && t27_value !== (t27_value = /*batch*/ ctx[5].summary.completedItems + "")) set_data_dev(t27, t27_value);
				if (dirty & /*actionHistory*/ 1 && t31_value !== (t31_value = /*batch*/ ctx[5].summary.failedItems + "")) set_data_dev(t31, t31_value);
				if (dirty & /*actionHistory*/ 1 && t35_value !== (t35_value = /*batch*/ ctx[5].summary.skippedItems + "")) set_data_dev(t35, t35_value);
				if (dirty & /*actionHistory*/ 1 && t41_value !== (t41_value = /*batch*/ ctx[5].options.aggressiveness + "")) set_data_dev(t41, t41_value);

				if (/*batch*/ ctx[5].options.blockCollabs) {
					if (if_block2) ; else {
						if_block2 = create_if_block_3$5(ctx);
						if_block2.c();
						if_block2.m(div25, t43);
					}
				} else if (if_block2) {
					if_block2.d(1);
					if_block2 = null;
				}

				if (/*batch*/ ctx[5].options.blockFeaturing) {
					if (if_block3) ; else {
						if_block3 = create_if_block_2$6(ctx);
						if_block3.c();
						if_block3.m(div25, t44);
					}
				} else if (if_block3) {
					if_block3.d(1);
					if_block3 = null;
				}

				if (/*batch*/ ctx[5].options.blockSongwriterOnly) {
					if (if_block4) ; else {
						if_block4 = create_if_block_1$9(ctx);
						if_block4.c();
						if_block4.m(div25, null);
					}
				} else if (if_block4) {
					if_block4.d(1);
					if_block4 = null;
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(li);
				}

				if (if_block0) if_block0.d();
				if (if_block1) if_block1.d();
				if (if_block2) if_block2.d();
				if (if_block3) if_block3.d();
				if (if_block4) if_block4.d();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_each_block$3.name,
			type: "each",
			source: "(72:8) {#each actionHistory as batch}",
			ctx
		});

		return block;
	}

	function create_fragment$9(ctx) {
		let div7;
		let div1;
		let div0;
		let h30;
		let t1;
		let p0;
		let t3;
		let t4;
		let t5;
		let div6;
		let div5;
		let div2;
		let svg;
		let path;
		let t6;
		let div4;
		let h31;
		let t8;
		let div3;
		let p1;
		let t10;
		let ul;
		let li0;
		let t12;
		let li1;
		let t14;
		let li2;
		let if_block0 = /*$canRollback*/ ctx[1] && create_if_block_6$5(ctx);

		function select_block_type(ctx, dirty) {
			if (/*actionHistory*/ ctx[0].length === 0) return create_if_block$9;
			return create_else_block$9;
		}

		let current_block_type = select_block_type(ctx);
		let if_block1 = current_block_type(ctx);

		const block = {
			c: function create() {
				div7 = element("div");
				div1 = element("div");
				div0 = element("div");
				h30 = element("h3");
				h30.textContent = "Action History";
				t1 = space();
				p0 = element("p");
				p0.textContent = "View and manage your past enforcement executions.";
				t3 = space();
				if (if_block0) if_block0.c();
				t4 = space();
				if_block1.c();
				t5 = space();
				div6 = element("div");
				div5 = element("div");
				div2 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t6 = space();
				div4 = element("div");
				h31 = element("h3");
				h31.textContent = "About Rollbacks";
				t8 = space();
				div3 = element("div");
				p1 = element("p");
				p1.textContent = "Rollback attempts to undo changes made during enforcement. Success depends on platform capabilities:";
				t10 = space();
				ul = element("ul");
				li0 = element("li");
				li0.textContent = "Re-adding liked songs and follows: Usually successful";
				t12 = space();
				li1 = element("li");
				li1.textContent = "Re-adding playlist tracks: May not preserve original order";
				t14 = space();
				li2 = element("li");
				li2.textContent = "Radio seed changes: May not be fully reversible";
				attr_dev(h30, "class", "text-lg font-medium text-gray-900");
				add_location(h30, file$9, 49, 6, 1428);
				attr_dev(p0, "class", "text-sm text-gray-500");
				add_location(p0, file$9, 50, 6, 1500);
				add_location(div0, file$9, 48, 4, 1416);
				attr_dev(div1, "class", "flex justify-between items-center");
				add_location(div1, file$9, 47, 2, 1364);
				attr_dev(path, "fill-rule", "evenodd");
				attr_dev(path, "d", "M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z");
				attr_dev(path, "clip-rule", "evenodd");
				add_location(path, file$9, 183, 10, 8282);
				attr_dev(svg, "class", "h-5 w-5 text-blue-400");
				attr_dev(svg, "viewBox", "0 0 20 20");
				attr_dev(svg, "fill", "currentColor");
				add_location(svg, file$9, 182, 8, 8196);
				attr_dev(div2, "class", "flex-shrink-0");
				add_location(div2, file$9, 181, 6, 8160);
				attr_dev(h31, "class", "text-sm font-medium text-blue-800");
				add_location(h31, file$9, 187, 8, 8525);
				add_location(p1, file$9, 191, 10, 8671);
				add_location(li0, file$9, 195, 12, 8875);
				add_location(li1, file$9, 196, 12, 8950);
				add_location(li2, file$9, 197, 12, 9030);
				attr_dev(ul, "class", "list-disc list-inside mt-1 space-y-1");
				add_location(ul, file$9, 194, 10, 8813);
				attr_dev(div3, "class", "mt-2 text-sm text-blue-700");
				add_location(div3, file$9, 190, 8, 8620);
				attr_dev(div4, "class", "ml-3");
				add_location(div4, file$9, 186, 6, 8498);
				attr_dev(div5, "class", "flex");
				add_location(div5, file$9, 180, 4, 8135);
				attr_dev(div6, "class", "bg-blue-50 border border-blue-200 rounded-md p-4");
				add_location(div6, file$9, 179, 2, 8068);
				attr_dev(div7, "class", "space-y-6");
				add_location(div7, file$9, 46, 0, 1338);
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				insert_dev(target, div7, anchor);
				append_dev(div7, div1);
				append_dev(div1, div0);
				append_dev(div0, h30);
				append_dev(div0, t1);
				append_dev(div0, p0);
				append_dev(div1, t3);
				if (if_block0) if_block0.m(div1, null);
				append_dev(div7, t4);
				if_block1.m(div7, null);
				append_dev(div7, t5);
				append_dev(div7, div6);
				append_dev(div6, div5);
				append_dev(div5, div2);
				append_dev(div2, svg);
				append_dev(svg, path);
				append_dev(div5, t6);
				append_dev(div5, div4);
				append_dev(div4, h31);
				append_dev(div4, t8);
				append_dev(div4, div3);
				append_dev(div3, p1);
				append_dev(div3, t10);
				append_dev(div3, ul);
				append_dev(ul, li0);
				append_dev(ul, t12);
				append_dev(ul, li1);
				append_dev(ul, t14);
				append_dev(ul, li2);
			},
			p: function update(ctx, [dirty]) {
				if (/*$canRollback*/ ctx[1]) {
					if (if_block0) ; else {
						if_block0 = create_if_block_6$5(ctx);
						if_block0.c();
						if_block0.m(div1, null);
					}
				} else if (if_block0) {
					if_block0.d(1);
					if_block0 = null;
				}

				if (current_block_type === (current_block_type = select_block_type(ctx)) && if_block1) {
					if_block1.p(ctx, dirty);
				} else {
					if_block1.d(1);
					if_block1 = current_block_type(ctx);

					if (if_block1) {
						if_block1.c();
						if_block1.m(div7, t5);
					}
				}
			},
			i: noop,
			o: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div7);
				}

				if (if_block0) if_block0.d();
				if_block1.d();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_fragment$9.name,
			type: "component",
			source: "",
			ctx
		});

		return block;
	}

	function getStatusColor(status) {
		switch (status) {
			case 'pending':
				return 'text-gray-600 bg-gray-100';
			case 'running':
				return 'text-blue-600 bg-blue-100';
			case 'completed':
				return 'text-green-600 bg-green-100';
			case 'failed':
				return 'text-red-600 bg-red-100';
			case 'cancelled':
				return 'text-yellow-600 bg-yellow-100';
			default:
				return 'text-gray-600 bg-gray-100';
		}
	}

	function formatDate$4(dateString) {
		return new Date(dateString).toLocaleString();
	}

	function getSuccessRate(batch) {
		const total = batch.summary.totalItems;
		const completed = batch.summary.completedItems;
		return total > 0 ? Math.round(completed / total * 100) : 0;
	}

	function instance$9($$self, $$props, $$invalidate) {
		let actionHistory;
		let $enforcementStore;
		let $canRollback;
		validate_store(enforcementStore, 'enforcementStore');
		component_subscribe($$self, enforcementStore, $$value => $$invalidate(3, $enforcementStore = $$value));
		validate_store(canRollback, 'canRollback');
		component_subscribe($$self, canRollback, $$value => $$invalidate(1, $canRollback = $$value));
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('ActionHistory', slots, []);

		async function rollbackBatch(batchId) {
			const confirmed = confirm('Are you sure you want to rollback this batch? This will attempt to undo the changes made during this enforcement.');

			if (confirmed) {
				const result = await enforcementActions.rollbackBatch(batchId);

				if (!result.success) {
					alert(`Rollback failed: ${result.message}`);
				}
			}
		}

		const writable_props = [];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<ActionHistory> was created with unknown prop '${key}'`);
		});

		const click_handler = batch => rollbackBatch(batch.id);

		$$self.$capture_state = () => ({
			enforcementActions,
			enforcementStore,
			canRollback,
			rollbackBatch,
			getStatusColor,
			formatDate: formatDate$4,
			getSuccessRate,
			actionHistory,
			$enforcementStore,
			$canRollback
		});

		$$self.$inject_state = $$props => {
			if ('actionHistory' in $$props) $$invalidate(0, actionHistory = $$props.actionHistory);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		$$self.$$.update = () => {
			if ($$self.$$.dirty & /*$enforcementStore*/ 8) {
				$$invalidate(0, actionHistory = $enforcementStore.actionHistory);
			}
		};

		return [actionHistory, $canRollback, rollbackBatch, $enforcementStore, click_handler];
	}

	class ActionHistory extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance$9, create_fragment$9, safe_not_equal, {});

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "ActionHistory",
				options,
				id: create_fragment$9.name
			});
		}
	}

	/* src/lib/components/EnforcementPlanning.svelte generated by Svelte v4.2.20 */
	const file$8 = "src/lib/components/EnforcementPlanning.svelte";

	// (37:2) {#if !$hasActiveSpotifyConnection || $dnpCount === 0}
	function create_if_block_6$4(ctx) {
		let div4;
		let div3;
		let div0;
		let svg;
		let path;
		let t0;
		let div2;
		let h3;
		let t2;
		let div1;
		let p;
		let t4;
		let ul;
		let t5;
		let if_block0 = !/*$hasActiveSpotifyConnection*/ ctx[1] && create_if_block_8$2(ctx);
		let if_block1 = /*$dnpCount*/ ctx[2] === 0 && create_if_block_7$4(ctx);

		const block = {
			c: function create() {
				div4 = element("div");
				div3 = element("div");
				div0 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				div2 = element("div");
				h3 = element("h3");
				h3.textContent = "Setup Required";
				t2 = space();
				div1 = element("div");
				p = element("p");
				p.textContent = "Before you can plan enforcement, you need:";
				t4 = space();
				ul = element("ul");
				if (if_block0) if_block0.c();
				t5 = space();
				if (if_block1) if_block1.c();
				attr_dev(path, "fill-rule", "evenodd");
				attr_dev(path, "d", "M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z");
				attr_dev(path, "clip-rule", "evenodd");
				add_location(path, file$8, 48, 12, 1502);
				attr_dev(svg, "class", "h-5 w-5 text-yellow-400");
				attr_dev(svg, "viewBox", "0 0 20 20");
				attr_dev(svg, "fill", "currentColor");
				add_location(svg, file$8, 47, 10, 1412);
				attr_dev(div0, "class", "flex-shrink-0");
				add_location(div0, file$8, 46, 8, 1374);
				attr_dev(h3, "class", "text-sm font-medium text-yellow-800");
				add_location(h3, file$8, 52, 10, 1834);
				add_location(p, file$8, 56, 12, 1991);
				attr_dev(ul, "class", "list-disc list-inside mt-1 space-y-1");
				add_location(ul, file$8, 57, 12, 2053);
				attr_dev(div1, "class", "mt-2 text-sm text-yellow-700");
				add_location(div1, file$8, 55, 10, 1936);
				attr_dev(div2, "class", "ml-3");
				add_location(div2, file$8, 51, 8, 1805);
				attr_dev(div3, "class", "flex");
				add_location(div3, file$8, 45, 6, 1347);
				attr_dev(div4, "class", "mb-6 bg-yellow-50 border border-yellow-200 rounded-md p-4");
				add_location(div4, file$8, 44, 4, 1269);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div4, anchor);
				append_dev(div4, div3);
				append_dev(div3, div0);
				append_dev(div0, svg);
				append_dev(svg, path);
				append_dev(div3, t0);
				append_dev(div3, div2);
				append_dev(div2, h3);
				append_dev(div2, t2);
				append_dev(div2, div1);
				append_dev(div1, p);
				append_dev(div1, t4);
				append_dev(div1, ul);
				if (if_block0) if_block0.m(ul, null);
				append_dev(ul, t5);
				if (if_block1) if_block1.m(ul, null);
			},
			p: function update(ctx, dirty) {
				if (!/*$hasActiveSpotifyConnection*/ ctx[1]) {
					if (if_block0) ; else {
						if_block0 = create_if_block_8$2(ctx);
						if_block0.c();
						if_block0.m(ul, t5);
					}
				} else if (if_block0) {
					if_block0.d(1);
					if_block0 = null;
				}

				if (/*$dnpCount*/ ctx[2] === 0) {
					if (if_block1) ; else {
						if_block1 = create_if_block_7$4(ctx);
						if_block1.c();
						if_block1.m(ul, null);
					}
				} else if (if_block1) {
					if_block1.d(1);
					if_block1 = null;
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div4);
				}

				if (if_block0) if_block0.d();
				if (if_block1) if_block1.d();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_6$4.name,
			type: "if",
			source: "(37:2) {#if !$hasActiveSpotifyConnection || $dnpCount === 0}",
			ctx
		});

		return block;
	}

	// (52:14) {#if !$hasActiveSpotifyConnection}
	function create_if_block_8$2(ctx) {
		let li;

		const block = {
			c: function create() {
				li = element("li");
				li.textContent = "Connect at least one streaming service (Spotify)";
				add_location(li, file$8, 59, 16, 2168);
			},
			m: function mount(target, anchor) {
				insert_dev(target, li, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(li);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_8$2.name,
			type: "if",
			source: "(52:14) {#if !$hasActiveSpotifyConnection}",
			ctx
		});

		return block;
	}

	// (55:14) {#if $dnpCount === 0}
	function create_if_block_7$4(ctx) {
		let li;

		const block = {
			c: function create() {
				li = element("li");
				li.textContent = "Add artists to your Do-Not-Play list";
				add_location(li, file$8, 62, 16, 2298);
			},
			m: function mount(target, anchor) {
				insert_dev(target, li, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(li);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_7$4.name,
			type: "if",
			source: "(55:14) {#if $dnpCount === 0}",
			ctx
		});

		return block;
	}

	// (166:36) 
	function create_if_block_5$4(ctx) {
		let actionhistory;
		let current;
		actionhistory = new ActionHistory({ $$inline: true });

		const block = {
			c: function create() {
				create_component(actionhistory.$$.fragment);
			},
			m: function mount(target, anchor) {
				mount_component(actionhistory, target, anchor);
				current = true;
			},
			p: noop,
			i: function intro(local) {
				if (current) return;
				transition_in(actionhistory.$$.fragment, local);
				current = true;
			},
			o: function outro(local) {
				transition_out(actionhistory.$$.fragment, local);
				current = false;
			},
			d: function destroy(detaching) {
				destroy_component(actionhistory, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_5$4.name,
			type: "if",
			source: "(166:36) ",
			ctx
		});

		return block;
	}

	// (164:36) 
	function create_if_block_4$4(ctx) {
		let enforcementexecution;
		let current;
		enforcementexecution = new EnforcementExecution({ $$inline: true });

		const block = {
			c: function create() {
				create_component(enforcementexecution.$$.fragment);
			},
			m: function mount(target, anchor) {
				mount_component(enforcementexecution, target, anchor);
				current = true;
			},
			p: noop,
			i: function intro(local) {
				if (current) return;
				transition_in(enforcementexecution.$$.fragment, local);
				current = true;
			},
			o: function outro(local) {
				transition_out(enforcementexecution.$$.fragment, local);
				current = false;
			},
			d: function destroy(detaching) {
				destroy_component(enforcementexecution, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_4$4.name,
			type: "if",
			source: "(164:36) ",
			ctx
		});

		return block;
	}

	// (91:2) {#if activeTab === 'plan'}
	function create_if_block$8(ctx) {
		let div1;
		let div0;
		let h3;
		let t1;
		let enforcementoptions;
		let t2;
		let current_block_type_index;
		let if_block0;
		let t3;
		let current;
		enforcementoptions = new EnforcementOptions({ $$inline: true });
		const if_block_creators = [create_if_block_2$5, create_else_block$8];
		const if_blocks = [];

		function select_block_type_1(ctx, dirty) {
			if (/*$hasActivePlan*/ ctx[3]) return 0;
			return 1;
		}

		current_block_type_index = select_block_type_1(ctx);
		if_block0 = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);
		let if_block1 = /*$enforcementStore*/ ctx[4].error && create_if_block_1$8(ctx);

		const block = {
			c: function create() {
				div1 = element("div");
				div0 = element("div");
				h3 = element("h3");
				h3.textContent = "Enforcement Options";
				t1 = space();
				create_component(enforcementoptions.$$.fragment);
				t2 = space();
				if_block0.c();
				t3 = space();
				if (if_block1) if_block1.c();
				attr_dev(h3, "class", "text-lg font-medium text-gray-900 mb-4");
				add_location(h3, file$8, 101, 8, 3734);
				attr_dev(div0, "class", "bg-white shadow rounded-lg p-6");
				add_location(div0, file$8, 100, 6, 3681);
				attr_dev(div1, "class", "space-y-6");
				add_location(div1, file$8, 98, 4, 3616);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div1, anchor);
				append_dev(div1, div0);
				append_dev(div0, h3);
				append_dev(div0, t1);
				mount_component(enforcementoptions, div0, null);
				append_dev(div1, t2);
				if_blocks[current_block_type_index].m(div1, null);
				append_dev(div1, t3);
				if (if_block1) if_block1.m(div1, null);
				current = true;
			},
			p: function update(ctx, dirty) {
				let previous_block_index = current_block_type_index;
				current_block_type_index = select_block_type_1(ctx);

				if (current_block_type_index === previous_block_index) {
					if_blocks[current_block_type_index].p(ctx, dirty);
				} else {
					group_outros();

					transition_out(if_blocks[previous_block_index], 1, 1, () => {
						if_blocks[previous_block_index] = null;
					});

					check_outros();
					if_block0 = if_blocks[current_block_type_index];

					if (!if_block0) {
						if_block0 = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);
						if_block0.c();
					} else {
						if_block0.p(ctx, dirty);
					}

					transition_in(if_block0, 1);
					if_block0.m(div1, t3);
				}

				if (/*$enforcementStore*/ ctx[4].error) {
					if (if_block1) {
						if_block1.p(ctx, dirty);
					} else {
						if_block1 = create_if_block_1$8(ctx);
						if_block1.c();
						if_block1.m(div1, null);
					}
				} else if (if_block1) {
					if_block1.d(1);
					if_block1 = null;
				}
			},
			i: function intro(local) {
				if (current) return;
				transition_in(enforcementoptions.$$.fragment, local);
				transition_in(if_block0);
				current = true;
			},
			o: function outro(local) {
				transition_out(enforcementoptions.$$.fragment, local);
				transition_out(if_block0);
				current = false;
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div1);
				}

				destroy_component(enforcementoptions);
				if_blocks[current_block_type_index].d();
				if (if_block1) if_block1.d();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block$8.name,
			type: "if",
			source: "(91:2) {#if activeTab === 'plan'}",
			ctx
		});

		return block;
	}

	// (113:6) {:else}
	function create_else_block$8(ctx) {
		let div;
		let h3;
		let t1;
		let p;
		let t3;
		let button;
		let button_disabled_value;
		let mounted;
		let dispose;

		function select_block_type_2(ctx, dirty) {
			if (/*$enforcementStore*/ ctx[4].isPlanning) return create_if_block_3$4;
			return create_else_block_1$4;
		}

		let current_block_type = select_block_type_2(ctx);
		let if_block = current_block_type(ctx);

		const block = {
			c: function create() {
				div = element("div");
				h3 = element("h3");
				h3.textContent = "Create Enforcement Plan";
				t1 = space();
				p = element("p");
				p.textContent = "Generate a dry-run preview to see what changes will be made to your music library.";
				t3 = space();
				button = element("button");
				if_block.c();
				attr_dev(h3, "class", "text-lg font-medium text-gray-900 mb-4");
				add_location(h3, file$8, 122, 10, 4495);
				attr_dev(p, "class", "text-sm text-gray-600 mb-4");
				add_location(p, file$8, 123, 10, 4585);
				button.disabled = button_disabled_value = /*$enforcementStore*/ ctx[4].isPlanning || !/*$hasActiveSpotifyConnection*/ ctx[1] || /*$dnpCount*/ ctx[2] === 0;
				attr_dev(button, "class", "inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed");
				add_location(button, file$8, 127, 10, 4755);
				attr_dev(div, "class", "bg-white shadow rounded-lg p-6");
				add_location(div, file$8, 121, 8, 4440);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, h3);
				append_dev(div, t1);
				append_dev(div, p);
				append_dev(div, t3);
				append_dev(div, button);
				if_block.m(button, null);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*createPlan*/ ctx[5], false, false, false, false);
					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (current_block_type !== (current_block_type = select_block_type_2(ctx))) {
					if_block.d(1);
					if_block = current_block_type(ctx);

					if (if_block) {
						if_block.c();
						if_block.m(button, null);
					}
				}

				if (dirty & /*$enforcementStore, $hasActiveSpotifyConnection, $dnpCount*/ 22 && button_disabled_value !== (button_disabled_value = /*$enforcementStore*/ ctx[4].isPlanning || !/*$hasActiveSpotifyConnection*/ ctx[1] || /*$dnpCount*/ ctx[2] === 0)) {
					prop_dev(button, "disabled", button_disabled_value);
				}
			},
			i: noop,
			o: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}

				if_block.d();
				mounted = false;
				dispose();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block$8.name,
			type: "else",
			source: "(113:6) {:else}",
			ctx
		});

		return block;
	}

	// (100:6) {#if $hasActivePlan}
	function create_if_block_2$5(ctx) {
		let div1;
		let div0;
		let h3;
		let t1;
		let button;
		let t3;
		let enforcementpreview;
		let current;
		let mounted;
		let dispose;
		enforcementpreview = new EnforcementPreview({ $$inline: true });

		const block = {
			c: function create() {
				div1 = element("div");
				div0 = element("div");
				h3 = element("h3");
				h3.textContent = "Enforcement Preview";
				t1 = space();
				button = element("button");
				button.textContent = "Clear Plan";
				t3 = space();
				create_component(enforcementpreview.$$.fragment);
				attr_dev(h3, "class", "text-lg font-medium text-gray-900");
				add_location(h3, file$8, 109, 12, 4046);
				attr_dev(button, "class", "text-sm text-gray-500 hover:text-gray-700");
				add_location(button, file$8, 110, 12, 4129);
				attr_dev(div0, "class", "flex justify-between items-center mb-4");
				add_location(div0, file$8, 108, 10, 3981);
				attr_dev(div1, "class", "bg-white shadow rounded-lg p-6");
				add_location(div1, file$8, 107, 8, 3926);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div1, anchor);
				append_dev(div1, div0);
				append_dev(div0, h3);
				append_dev(div0, t1);
				append_dev(div0, button);
				append_dev(div1, t3);
				mount_component(enforcementpreview, div1, null);
				current = true;

				if (!mounted) {
					dispose = listen_dev(button, "click", /*click_handler_3*/ ctx[10], false, false, false, false);
					mounted = true;
				}
			},
			p: noop,
			i: function intro(local) {
				if (current) return;
				transition_in(enforcementpreview.$$.fragment, local);
				current = true;
			},
			o: function outro(local) {
				transition_out(enforcementpreview.$$.fragment, local);
				current = false;
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div1);
				}

				destroy_component(enforcementpreview);
				mounted = false;
				dispose();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_2$5.name,
			type: "if",
			source: "(100:6) {#if $hasActivePlan}",
			ctx
		});

		return block;
	}

	// (132:12) {:else}
	function create_else_block_1$4(ctx) {
		let svg;
		let path;
		let t;

		const block = {
			c: function create() {
				svg = svg_element("svg");
				path = svg_element("path");
				t = text("\n              Create Enforcement Plan");
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2");
				add_location(path, file$8, 140, 16, 5875);
				attr_dev(svg, "class", "-ml-1 mr-2 h-5 w-5");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$8, 139, 14, 5772);
			},
			m: function mount(target, anchor) {
				insert_dev(target, svg, anchor);
				append_dev(svg, path);
				insert_dev(target, t, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(svg);
					detach_dev(t);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block_1$4.name,
			type: "else",
			source: "(132:12) {:else}",
			ctx
		});

		return block;
	}

	// (126:12) {#if $enforcementStore.isPlanning}
	function create_if_block_3$4(ctx) {
		let svg;
		let circle;
		let path;
		let t;

		const block = {
			c: function create() {
				svg = svg_element("svg");
				circle = svg_element("circle");
				path = svg_element("path");
				t = text("\n              Creating Plan...");
				attr_dev(circle, "class", "opacity-25");
				attr_dev(circle, "cx", "12");
				attr_dev(circle, "cy", "12");
				attr_dev(circle, "r", "10");
				attr_dev(circle, "stroke", "currentColor");
				attr_dev(circle, "stroke-width", "4");
				add_location(circle, file$8, 134, 16, 5402);
				attr_dev(path, "class", "opacity-75");
				attr_dev(path, "fill", "currentColor");
				attr_dev(path, "d", "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z");
				add_location(path, file$8, 135, 16, 5517);
				attr_dev(svg, "class", "animate-spin -ml-1 mr-2 h-4 w-4 text-white");
				attr_dev(svg, "xmlns", "http://www.w3.org/2000/svg");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				add_location(svg, file$8, 133, 14, 5262);
			},
			m: function mount(target, anchor) {
				insert_dev(target, svg, anchor);
				append_dev(svg, circle);
				append_dev(svg, path);
				insert_dev(target, t, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(svg);
					detach_dev(t);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_3$4.name,
			type: "if",
			source: "(126:12) {#if $enforcementStore.isPlanning}",
			ctx
		});

		return block;
	}

	// (143:6) {#if $enforcementStore.error}
	function create_if_block_1$8(ctx) {
		let div3;
		let div2;
		let div0;
		let svg;
		let path;
		let t0;
		let div1;
		let p;
		let t1_value = /*$enforcementStore*/ ctx[4].error + "";
		let t1;
		let t2;
		let button;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				div3 = element("div");
				div2 = element("div");
				div0 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				div1 = element("div");
				p = element("p");
				t1 = text(t1_value);
				t2 = space();
				button = element("button");
				button.textContent = "Dismiss";
				attr_dev(path, "fill-rule", "evenodd");
				attr_dev(path, "d", "M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z");
				attr_dev(path, "clip-rule", "evenodd");
				add_location(path, file$8, 154, 16, 6512);
				attr_dev(svg, "class", "h-5 w-5 text-red-400");
				attr_dev(svg, "viewBox", "0 0 20 20");
				attr_dev(svg, "fill", "currentColor");
				add_location(svg, file$8, 153, 14, 6421);
				attr_dev(div0, "class", "flex-shrink-0");
				add_location(div0, file$8, 152, 12, 6379);
				attr_dev(p, "class", "text-sm text-red-800");
				add_location(p, file$8, 158, 14, 6866);
				attr_dev(button, "class", "mt-2 text-sm text-red-600 hover:text-red-500");
				add_location(button, file$8, 159, 14, 6942);
				attr_dev(div1, "class", "ml-3");
				add_location(div1, file$8, 157, 12, 6833);
				attr_dev(div2, "class", "flex");
				add_location(div2, file$8, 151, 10, 6348);
				attr_dev(div3, "class", "bg-red-50 border border-red-200 rounded-md p-4");
				add_location(div3, file$8, 150, 8, 6277);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div3, anchor);
				append_dev(div3, div2);
				append_dev(div2, div0);
				append_dev(div0, svg);
				append_dev(svg, path);
				append_dev(div2, t0);
				append_dev(div2, div1);
				append_dev(div1, p);
				append_dev(p, t1);
				append_dev(div1, t2);
				append_dev(div1, button);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*click_handler_4*/ ctx[11], false, false, false, false);
					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (dirty & /*$enforcementStore*/ 16 && t1_value !== (t1_value = /*$enforcementStore*/ ctx[4].error + "")) set_data_dev(t1, t1_value);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div3);
				}

				mounted = false;
				dispose();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_1$8.name,
			type: "if",
			source: "(143:6) {#if $enforcementStore.error}",
			ctx
		});

		return block;
	}

	function create_fragment$8(ctx) {
		let div2;
		let div0;
		let h2;
		let t1;
		let p;
		let t3;
		let t4;
		let div1;
		let nav;
		let button0;
		let t5;
		let button0_class_value;
		let t6;
		let button1;
		let t7;
		let button1_class_value;
		let button1_disabled_value;
		let t8;
		let button2;
		let t9;
		let button2_class_value;
		let t10;
		let current_block_type_index;
		let if_block1;
		let current;
		let mounted;
		let dispose;
		let if_block0 = (!/*$hasActiveSpotifyConnection*/ ctx[1] || /*$dnpCount*/ ctx[2] === 0) && create_if_block_6$4(ctx);
		const if_block_creators = [create_if_block$8, create_if_block_4$4, create_if_block_5$4];
		const if_blocks = [];

		function select_block_type(ctx, dirty) {
			if (/*activeTab*/ ctx[0] === 'plan') return 0;
			if (/*activeTab*/ ctx[0] === 'execute') return 1;
			if (/*activeTab*/ ctx[0] === 'history') return 2;
			return -1;
		}

		if (~(current_block_type_index = select_block_type(ctx))) {
			if_block1 = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);
		}

		const block = {
			c: function create() {
				div2 = element("div");
				div0 = element("div");
				h2 = element("h2");
				h2.textContent = "Enforcement Planning";
				t1 = space();
				p = element("p");
				p.textContent = "Plan and execute blocklist enforcement across your connected streaming services.";
				t3 = space();
				if (if_block0) if_block0.c();
				t4 = space();
				div1 = element("div");
				nav = element("nav");
				button0 = element("button");
				t5 = text("Plan Enforcement");
				t6 = space();
				button1 = element("button");
				t7 = text("Execute Plan");
				t8 = space();
				button2 = element("button");
				t9 = text("Action History");
				t10 = space();
				if (if_block1) if_block1.c();
				attr_dev(h2, "class", "text-2xl font-bold text-gray-900");
				add_location(h2, file$8, 36, 4, 958);
				attr_dev(p, "class", "mt-1 text-sm text-gray-600");
				add_location(p, file$8, 37, 4, 1033);
				attr_dev(div0, "class", "mb-6");
				add_location(div0, file$8, 35, 2, 935);

				attr_dev(button0, "class", button0_class_value = "py-4 px-1 border-b-2 font-medium text-sm " + (/*activeTab*/ ctx[0] === 'plan'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'));

				add_location(button0, file$8, 74, 6, 2586);

				attr_dev(button1, "class", button1_class_value = "py-4 px-1 border-b-2 font-medium text-sm " + (/*activeTab*/ ctx[0] === 'execute'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'));

				button1.disabled = button1_disabled_value = !/*$hasActivePlan*/ ctx[3];
				add_location(button1, file$8, 80, 6, 2892);

				attr_dev(button2, "class", button2_class_value = "py-4 px-1 border-b-2 font-medium text-sm " + (/*activeTab*/ ctx[0] === 'history'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'));

				add_location(button2, file$8, 87, 6, 3235);
				attr_dev(nav, "class", "flex space-x-8 px-6");
				attr_dev(nav, "aria-label", "Tabs");
				add_location(nav, file$8, 73, 4, 2528);
				attr_dev(div1, "class", "bg-white shadow-sm rounded-lg mb-6");
				add_location(div1, file$8, 72, 2, 2475);
				attr_dev(div2, "class", "px-4 py-6 sm:px-0");
				add_location(div2, file$8, 34, 0, 901);
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				insert_dev(target, div2, anchor);
				append_dev(div2, div0);
				append_dev(div0, h2);
				append_dev(div0, t1);
				append_dev(div0, p);
				append_dev(div2, t3);
				if (if_block0) if_block0.m(div2, null);
				append_dev(div2, t4);
				append_dev(div2, div1);
				append_dev(div1, nav);
				append_dev(nav, button0);
				append_dev(button0, t5);
				append_dev(nav, t6);
				append_dev(nav, button1);
				append_dev(button1, t7);
				append_dev(nav, t8);
				append_dev(nav, button2);
				append_dev(button2, t9);
				append_dev(div2, t10);

				if (~current_block_type_index) {
					if_blocks[current_block_type_index].m(div2, null);
				}

				current = true;

				if (!mounted) {
					dispose = [
						listen_dev(button0, "click", /*click_handler*/ ctx[7], false, false, false, false),
						listen_dev(button1, "click", /*click_handler_1*/ ctx[8], false, false, false, false),
						listen_dev(button2, "click", /*click_handler_2*/ ctx[9], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, [dirty]) {
				if (!/*$hasActiveSpotifyConnection*/ ctx[1] || /*$dnpCount*/ ctx[2] === 0) {
					if (if_block0) {
						if_block0.p(ctx, dirty);
					} else {
						if_block0 = create_if_block_6$4(ctx);
						if_block0.c();
						if_block0.m(div2, t4);
					}
				} else if (if_block0) {
					if_block0.d(1);
					if_block0 = null;
				}

				if (!current || dirty & /*activeTab*/ 1 && button0_class_value !== (button0_class_value = "py-4 px-1 border-b-2 font-medium text-sm " + (/*activeTab*/ ctx[0] === 'plan'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'))) {
					attr_dev(button0, "class", button0_class_value);
				}

				if (!current || dirty & /*activeTab*/ 1 && button1_class_value !== (button1_class_value = "py-4 px-1 border-b-2 font-medium text-sm " + (/*activeTab*/ ctx[0] === 'execute'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'))) {
					attr_dev(button1, "class", button1_class_value);
				}

				if (!current || dirty & /*$hasActivePlan*/ 8 && button1_disabled_value !== (button1_disabled_value = !/*$hasActivePlan*/ ctx[3])) {
					prop_dev(button1, "disabled", button1_disabled_value);
				}

				if (!current || dirty & /*activeTab*/ 1 && button2_class_value !== (button2_class_value = "py-4 px-1 border-b-2 font-medium text-sm " + (/*activeTab*/ ctx[0] === 'history'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'))) {
					attr_dev(button2, "class", button2_class_value);
				}

				let previous_block_index = current_block_type_index;
				current_block_type_index = select_block_type(ctx);

				if (current_block_type_index === previous_block_index) {
					if (~current_block_type_index) {
						if_blocks[current_block_type_index].p(ctx, dirty);
					}
				} else {
					if (if_block1) {
						group_outros();

						transition_out(if_blocks[previous_block_index], 1, 1, () => {
							if_blocks[previous_block_index] = null;
						});

						check_outros();
					}

					if (~current_block_type_index) {
						if_block1 = if_blocks[current_block_type_index];

						if (!if_block1) {
							if_block1 = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);
							if_block1.c();
						} else {
							if_block1.p(ctx, dirty);
						}

						transition_in(if_block1, 1);
						if_block1.m(div2, null);
					} else {
						if_block1 = null;
					}
				}
			},
			i: function intro(local) {
				if (current) return;
				transition_in(if_block1);
				current = true;
			},
			o: function outro(local) {
				transition_out(if_block1);
				current = false;
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div2);
				}

				if (if_block0) if_block0.d();

				if (~current_block_type_index) {
					if_blocks[current_block_type_index].d();
				}

				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_fragment$8.name,
			type: "component",
			source: "",
			ctx
		});

		return block;
	}

	function instance$8($$self, $$props, $$invalidate) {
		let $hasActiveSpotifyConnection;
		let $dnpCount;
		let $hasActivePlan;
		let $enforcementStore;
		validate_store(hasActiveSpotifyConnection, 'hasActiveSpotifyConnection');
		component_subscribe($$self, hasActiveSpotifyConnection, $$value => $$invalidate(1, $hasActiveSpotifyConnection = $$value));
		validate_store(dnpCount, 'dnpCount');
		component_subscribe($$self, dnpCount, $$value => $$invalidate(2, $dnpCount = $$value));
		validate_store(hasActivePlan, 'hasActivePlan');
		component_subscribe($$self, hasActivePlan, $$value => $$invalidate(3, $hasActivePlan = $$value));
		validate_store(enforcementStore, 'enforcementStore');
		component_subscribe($$self, enforcementStore, $$value => $$invalidate(4, $enforcementStore = $$value));
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('EnforcementPlanning', slots, []);
		let activeTab = 'plan';

		onMount(() => {
			enforcementActions.fetchActionHistory();
		});

		async function createPlan() {
			const providers = [];

			if ($hasActiveSpotifyConnection) {
				providers.push('spotify');
			}

			if (providers.length === 0) {
				return;
			}

			await enforcementActions.createPlan(providers, true);
		}

		function setActiveTab(tab) {
			$$invalidate(0, activeTab = tab);
		}

		const writable_props = [];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<EnforcementPlanning> was created with unknown prop '${key}'`);
		});

		const click_handler = () => setActiveTab('plan');
		const click_handler_1 = () => setActiveTab('execute');
		const click_handler_2 = () => setActiveTab('history');
		const click_handler_3 = () => enforcementActions.clearPlan();
		const click_handler_4 = () => enforcementActions.clearError();

		$$self.$capture_state = () => ({
			onMount,
			enforcementActions,
			enforcementStore,
			hasActivePlan,
			hasActiveSpotifyConnection,
			dnpCount,
			EnforcementOptions,
			EnforcementPreview,
			EnforcementExecution,
			ActionHistory,
			activeTab,
			createPlan,
			setActiveTab,
			$hasActiveSpotifyConnection,
			$dnpCount,
			$hasActivePlan,
			$enforcementStore
		});

		$$self.$inject_state = $$props => {
			if ('activeTab' in $$props) $$invalidate(0, activeTab = $$props.activeTab);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		return [
			activeTab,
			$hasActiveSpotifyConnection,
			$dnpCount,
			$hasActivePlan,
			$enforcementStore,
			createPlan,
			setActiveTab,
			click_handler,
			click_handler_1,
			click_handler_2,
			click_handler_3,
			click_handler_4
		];
	}

	class EnforcementPlanning extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance$8, create_fragment$8, safe_not_equal, {});

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "EnforcementPlanning",
				options,
				id: create_fragment$8.name
			});
		}
	}

	const initialState = {
	    lists: [],
	    currentList: null,
	    subscriptions: [],
	    subscriptionImpact: null,
	    isLoading: false,
	    isLoadingList: false,
	    isLoadingImpact: false,
	    searchQuery: '',
	    sortBy: 'updated_at',
	    sortOrder: 'desc',
	    error: null,
	};
	const communityStore = writable(initialState);
	const filteredLists = derived(communityStore, ($community) => {
	    let filtered = $community.lists;
	    // Apply search filter
	    if ($community.searchQuery.trim()) {
	        const query = $community.searchQuery.toLowerCase();
	        filtered = filtered.filter(list => list.name.toLowerCase().includes(query) ||
	            list.description.toLowerCase().includes(query) ||
	            list.criteria.toLowerCase().includes(query));
	    }
	    // Apply sorting
	    filtered.sort((a, b) => {
	        let aValue = a[$community.sortBy];
	        let bValue = b[$community.sortBy];
	        if ($community.sortBy === 'created_at' || $community.sortBy === 'updated_at') {
	            aValue = new Date(aValue).getTime();
	            bValue = new Date(bValue).getTime();
	        }
	        if (typeof aValue === 'string') {
	            aValue = aValue.toLowerCase();
	            bValue = bValue.toLowerCase();
	        }
	        const comparison = aValue < bValue ? -1 : aValue > bValue ? 1 : 0;
	        return $community.sortOrder === 'asc' ? comparison : -comparison;
	    });
	    return filtered;
	});
	const subscribedListIds = derived(communityStore, ($community) => new Set($community.subscriptions.map(sub => sub.list_id)));
	derived(subscribedListIds, ($subscribedIds) => (listId) => $subscribedIds.has(listId));
	// Community actions
	const communityActions = {
	    fetchLists: async () => {
	        communityStore.update(state => ({ ...state, isLoading: true, error: null }));
	        try {
	            const token = localStorage.getItem('auth_token');
	            const response = await fetch('http://localhost:3000/api/v1/community/lists', {
	                headers: {
	                    'Authorization': `Bearer ${token}`,
	                },
	            });
	            const result = await response.json();
	            if (result.success) {
	                communityStore.update(state => ({
	                    ...state,
	                    lists: result.data,
	                    isLoading: false,
	                }));
	            }
	            else {
	                communityStore.update(state => ({
	                    ...state,
	                    error: result.message,
	                    isLoading: false,
	                }));
	            }
	        }
	        catch (error) {
	            communityStore.update(state => ({
	                ...state,
	                error: 'Failed to fetch community lists',
	                isLoading: false,
	            }));
	        }
	    },
	    fetchListDetails: async (listId) => {
	        communityStore.update(state => ({ ...state, isLoadingList: true, error: null }));
	        try {
	            const token = localStorage.getItem('auth_token');
	            const response = await fetch(`http://localhost:3000/api/v1/community/lists/${listId}/artists`, {
	                headers: {
	                    'Authorization': `Bearer ${token}`,
	                },
	            });
	            const result = await response.json();
	            if (result.success) {
	                communityStore.update(state => ({
	                    ...state,
	                    currentList: result.data,
	                    isLoadingList: false,
	                }));
	            }
	            else {
	                communityStore.update(state => ({
	                    ...state,
	                    error: result.message,
	                    isLoadingList: false,
	                }));
	            }
	        }
	        catch (error) {
	            communityStore.update(state => ({
	                ...state,
	                error: 'Failed to fetch list details',
	                isLoadingList: false,
	            }));
	        }
	    },
	    fetchSubscriptions: async () => {
	        try {
	            const token = localStorage.getItem('auth_token');
	            const response = await fetch('http://localhost:3000/api/v1/community/subscriptions', {
	                headers: {
	                    'Authorization': `Bearer ${token}`,
	                },
	            });
	            const result = await response.json();
	            if (result.success) {
	                communityStore.update(state => ({
	                    ...state,
	                    subscriptions: result.data,
	                }));
	            }
	        }
	        catch (error) {
	            console.error('Failed to fetch subscriptions:', error);
	        }
	    },
	    getSubscriptionImpact: async (listId) => {
	        communityStore.update(state => ({ ...state, isLoadingImpact: true, error: null }));
	        try {
	            const token = localStorage.getItem('auth_token');
	            const response = await fetch(`http://localhost:3000/api/v1/community/lists/${listId}/impact`, {
	                headers: {
	                    'Authorization': `Bearer ${token}`,
	                },
	            });
	            const result = await response.json();
	            if (result.success) {
	                communityStore.update(state => ({
	                    ...state,
	                    subscriptionImpact: result.data,
	                    isLoadingImpact: false,
	                }));
	                return { success: true, data: result.data };
	            }
	            else {
	                communityStore.update(state => ({
	                    ...state,
	                    error: result.message,
	                    isLoadingImpact: false,
	                }));
	                return { success: false, message: result.message };
	            }
	        }
	        catch (error) {
	            communityStore.update(state => ({
	                ...state,
	                error: 'Failed to get subscription impact',
	                isLoadingImpact: false,
	            }));
	            return { success: false, message: 'Failed to get subscription impact' };
	        }
	    },
	    subscribe: async (listId, versionPinned, autoUpdate = true) => {
	        try {
	            const token = localStorage.getItem('auth_token');
	            const response = await fetch(`http://localhost:3000/api/v1/community/lists/${listId}/subscribe`, {
	                method: 'POST',
	                headers: {
	                    'Authorization': `Bearer ${token}`,
	                    'Content-Type': 'application/json',
	                },
	                body: JSON.stringify({
	                    version_pinned: versionPinned,
	                    auto_update: autoUpdate,
	                }),
	            });
	            const result = await response.json();
	            if (result.success) {
	                // Refresh subscriptions
	                await communityActions.fetchSubscriptions();
	                return { success: true };
	            }
	            else {
	                return { success: false, message: result.message };
	            }
	        }
	        catch (error) {
	            return { success: false, message: 'Failed to subscribe to list' };
	        }
	    },
	    unsubscribe: async (listId) => {
	        try {
	            const token = localStorage.getItem('auth_token');
	            const response = await fetch(`http://localhost:3000/api/v1/community/lists/${listId}/unsubscribe`, {
	                method: 'POST',
	                headers: {
	                    'Authorization': `Bearer ${token}`,
	                },
	            });
	            const result = await response.json();
	            if (result.success) {
	                // Refresh subscriptions
	                await communityActions.fetchSubscriptions();
	                return { success: true };
	            }
	            else {
	                return { success: false, message: result.message };
	            }
	        }
	        catch (error) {
	            return { success: false, message: 'Failed to unsubscribe from list' };
	        }
	    },
	    updateSubscription: async (listId, versionPinned, autoUpdate) => {
	        try {
	            const token = localStorage.getItem('auth_token');
	            const response = await fetch(`http://localhost:3000/api/v1/community/lists/${listId}/subscription`, {
	                method: 'PUT',
	                headers: {
	                    'Authorization': `Bearer ${token}`,
	                    'Content-Type': 'application/json',
	                },
	                body: JSON.stringify({
	                    version_pinned: versionPinned,
	                    auto_update: autoUpdate,
	                }),
	            });
	            const result = await response.json();
	            if (result.success) {
	                // Refresh subscriptions
	                await communityActions.fetchSubscriptions();
	                return { success: true };
	            }
	            else {
	                return { success: false, message: result.message };
	            }
	        }
	        catch (error) {
	            return { success: false, message: 'Failed to update subscription' };
	        }
	    },
	    createList: async (listData) => {
	        try {
	            const token = localStorage.getItem('auth_token');
	            const response = await fetch('http://localhost:3000/api/v1/community/lists', {
	                method: 'POST',
	                headers: {
	                    'Authorization': `Bearer ${token}`,
	                    'Content-Type': 'application/json',
	                },
	                body: JSON.stringify(listData),
	            });
	            const result = await response.json();
	            if (result.success) {
	                // Refresh lists
	                await communityActions.fetchLists();
	                return { success: true, data: result.data };
	            }
	            else {
	                return { success: false, message: result.message };
	            }
	        }
	        catch (error) {
	            return { success: false, message: 'Failed to create community list' };
	        }
	    },
	    updateSearch: (query) => {
	        communityStore.update(state => ({ ...state, searchQuery: query }));
	    },
	    updateSort: (sortBy, sortOrder) => {
	        communityStore.update(state => ({ ...state, sortBy, sortOrder }));
	    },
	    clearCurrentList: () => {
	        communityStore.update(state => ({ ...state, currentList: null }));
	    },
	    clearError: () => {
	        communityStore.update(state => ({ ...state, error: null }));
	    },
	};

	/* src/lib/components/CommunityListCard.svelte generated by Svelte v4.2.20 */
	const file$7 = "src/lib/components/CommunityListCard.svelte";

	// (54:8) {#if isSubscribed}
	function create_if_block_2$4(ctx) {
		let span;

		const block = {
			c: function create() {
				span = element("span");
				span.textContent = "Subscribed";
				attr_dev(span, "class", "ml-2 inline-flex items-center px-2 py-0.5 rounded text-xs font-medium text-green-800 bg-green-100");
				add_location(span, file$7, 62, 10, 1914);
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
			id: create_if_block_2$4.name,
			type: "if",
			source: "(54:8) {#if isSubscribed}",
			ctx
		});

		return block;
	}

	// (96:6) {#if list.governance_url}
	function create_if_block_1$7(ctx) {
		let t0;
		let a;
		let t1;
		let a_href_value;

		const block = {
			c: function create() {
				t0 = text("• ");
				a = element("a");
				t1 = text("Governance");
				attr_dev(a, "href", a_href_value = /*list*/ ctx[0].governance_url);
				attr_dev(a, "target", "_blank");
				attr_dev(a, "class", "text-indigo-600 hover:text-indigo-500");
				add_location(a, file$7, 104, 10, 3848);
			},
			m: function mount(target, anchor) {
				insert_dev(target, t0, anchor);
				insert_dev(target, a, anchor);
				append_dev(a, t1);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*list*/ 1 && a_href_value !== (a_href_value = /*list*/ ctx[0].governance_url)) {
					attr_dev(a, "href", a_href_value);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(t0);
					detach_dev(a);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_1$7.name,
			type: "if",
			source: "(96:6) {#if list.governance_url}",
			ctx
		});

		return block;
	}

	// (122:8) {:else}
	function create_else_block$7(ctx) {
		let svg;
		let path;
		let t;

		const block = {
			c: function create() {
				svg = svg_element("svg");
				path = svg_element("path");
				t = text("\n          Subscribe");
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M12 6v6m0 0v6m0-6h6m-6 0H6");
				add_location(path, file$7, 131, 12, 5059);
				attr_dev(svg, "class", "-ml-0.5 mr-2 h-4 w-4");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$7, 130, 10, 4958);
			},
			m: function mount(target, anchor) {
				insert_dev(target, svg, anchor);
				append_dev(svg, path);
				insert_dev(target, t, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(svg);
					detach_dev(t);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block$7.name,
			type: "else",
			source: "(122:8) {:else}",
			ctx
		});

		return block;
	}

	// (117:8) {#if isSubscribed}
	function create_if_block$7(ctx) {
		let svg;
		let path;
		let t;

		const block = {
			c: function create() {
				svg = svg_element("svg");
				path = svg_element("path");
				t = text("\n          Unsubscribe");
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M6 18L18 6M6 6l12 12");
				add_location(path, file$7, 126, 12, 4795);
				attr_dev(svg, "class", "-ml-0.5 mr-2 h-4 w-4");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$7, 125, 10, 4694);
			},
			m: function mount(target, anchor) {
				insert_dev(target, svg, anchor);
				append_dev(svg, path);
				insert_dev(target, t, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(svg);
					detach_dev(t);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block$7.name,
			type: "if",
			source: "(117:8) {#if isSubscribed}",
			ctx
		});

		return block;
	}

	function create_fragment$7(ctx) {
		let div9;
		let div6;
		let div1;
		let div0;
		let h3;
		let t0_value = /*list*/ ctx[0].name + "";
		let t0;
		let t1;
		let t2;
		let span0;
		let t3_value = /*list*/ ctx[0].update_cadence + "";
		let t3;
		let span0_class_value;
		let t4;
		let p0;
		let t5_value = /*list*/ ctx[0].description + "";
		let t5;
		let t6;
		let div2;
		let h4;
		let t8;
		let p1;
		let t9_value = /*list*/ ctx[0].criteria + "";
		let t9;
		let t10;
		let div4;
		let div3;
		let span1;
		let svg0;
		let path0;
		let t11;
		let t12_value = (/*list*/ ctx[0].artist_count || 0) + "";
		let t12;
		let t13;
		let t14;
		let span2;
		let svg1;
		let path1;
		let t15;
		let t16_value = (/*list*/ ctx[0].subscriber_count || 0) + "";
		let t16;
		let t17;
		let t18;
		let span3;
		let t19;
		let t20_value = /*list*/ ctx[0].version + "";
		let t20;
		let t21;
		let div5;
		let t22;
		let t23_value = formatDate$3(/*list*/ ctx[0].updated_at) + "";
		let t23;
		let t24;
		let t25;
		let div8;
		let div7;
		let button0;
		let t27;
		let button1;
		let button1_class_value;
		let mounted;
		let dispose;
		let if_block0 = /*isSubscribed*/ ctx[1] && create_if_block_2$4(ctx);
		let if_block1 = /*list*/ ctx[0].governance_url && create_if_block_1$7(ctx);

		function select_block_type(ctx, dirty) {
			if (/*isSubscribed*/ ctx[1]) return create_if_block$7;
			return create_else_block$7;
		}

		let current_block_type = select_block_type(ctx);
		let if_block2 = current_block_type(ctx);

		const block = {
			c: function create() {
				div9 = element("div");
				div6 = element("div");
				div1 = element("div");
				div0 = element("div");
				h3 = element("h3");
				t0 = text(t0_value);
				t1 = space();
				if (if_block0) if_block0.c();
				t2 = space();
				span0 = element("span");
				t3 = text(t3_value);
				t4 = space();
				p0 = element("p");
				t5 = text(t5_value);
				t6 = space();
				div2 = element("div");
				h4 = element("h4");
				h4.textContent = "Criteria";
				t8 = space();
				p1 = element("p");
				t9 = text(t9_value);
				t10 = space();
				div4 = element("div");
				div3 = element("div");
				span1 = element("span");
				svg0 = svg_element("svg");
				path0 = svg_element("path");
				t11 = space();
				t12 = text(t12_value);
				t13 = text(" artists");
				t14 = space();
				span2 = element("span");
				svg1 = svg_element("svg");
				path1 = svg_element("path");
				t15 = space();
				t16 = text(t16_value);
				t17 = text(" subscribers");
				t18 = space();
				span3 = element("span");
				t19 = text("v");
				t20 = text(t20_value);
				t21 = space();
				div5 = element("div");
				t22 = text("Updated ");
				t23 = text(t23_value);
				t24 = space();
				if (if_block1) if_block1.c();
				t25 = space();
				div8 = element("div");
				div7 = element("div");
				button0 = element("button");
				button0.textContent = "View Details";
				t27 = space();
				button1 = element("button");
				if_block2.c();
				attr_dev(h3, "class", "text-lg font-medium text-gray-900 truncate");
				add_location(h3, file$7, 58, 8, 1785);
				attr_dev(div0, "class", "flex items-center");
				add_location(div0, file$7, 57, 6, 1745);
				attr_dev(span0, "class", span0_class_value = "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium " + getUpdateCadenceColor$1(/*list*/ ctx[0].update_cadence));
				add_location(span0, file$7, 67, 6, 2101);
				attr_dev(div1, "class", "flex items-center justify-between");
				add_location(div1, file$7, 56, 4, 1691);
				attr_dev(p0, "class", "mt-2 text-sm text-gray-600 line-clamp-2");
				add_location(p0, file$7, 72, 4, 2290);
				attr_dev(h4, "class", "text-xs font-medium text-gray-900 uppercase tracking-wide");
				add_location(h4, file$7, 77, 6, 2410);
				attr_dev(p1, "class", "mt-1 text-sm text-gray-600 line-clamp-2");
				add_location(p1, file$7, 78, 6, 2500);
				attr_dev(div2, "class", "mt-3");
				add_location(div2, file$7, 76, 4, 2385);
				attr_dev(path0, "stroke-linecap", "round");
				attr_dev(path0, "stroke-linejoin", "round");
				attr_dev(path0, "stroke-width", "2");
				attr_dev(path0, "d", "M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z");
				add_location(path0, file$7, 87, 12, 2869);
				attr_dev(svg0, "class", "inline h-4 w-4 text-gray-400 mr-1");
				attr_dev(svg0, "fill", "none");
				attr_dev(svg0, "viewBox", "0 0 24 24");
				attr_dev(svg0, "stroke", "currentColor");
				add_location(svg0, file$7, 86, 10, 2755);
				add_location(span1, file$7, 85, 8, 2738);
				attr_dev(path1, "stroke-linecap", "round");
				attr_dev(path1, "stroke-linejoin", "round");
				attr_dev(path1, "stroke-width", "2");
				attr_dev(path1, "d", "M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z");
				add_location(path1, file$7, 93, 12, 3229);
				attr_dev(svg1, "class", "inline h-4 w-4 text-gray-400 mr-1");
				attr_dev(svg1, "fill", "none");
				attr_dev(svg1, "viewBox", "0 0 24 24");
				attr_dev(svg1, "stroke", "currentColor");
				add_location(svg1, file$7, 92, 10, 3115);
				add_location(span2, file$7, 91, 8, 3098);
				attr_dev(div3, "class", "flex items-center space-x-4");
				add_location(div3, file$7, 84, 6, 2688);
				add_location(span3, file$7, 98, 6, 3672);
				attr_dev(div4, "class", "mt-4 flex items-center justify-between text-sm text-gray-500");
				add_location(div4, file$7, 83, 4, 2607);
				attr_dev(div5, "class", "mt-4 text-xs text-gray-400");
				add_location(div5, file$7, 101, 4, 3721);
				attr_dev(div6, "class", "p-6");
				add_location(div6, file$7, 55, 2, 1669);
				attr_dev(button0, "class", "text-sm text-indigo-600 hover:text-indigo-500 font-medium");
				add_location(button0, file$7, 111, 6, 4085);

				attr_dev(button1, "class", button1_class_value = "inline-flex items-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md " + (/*isSubscribed*/ ctx[1]
				? 'text-red-700 bg-red-100 hover:bg-red-200 focus:ring-red-500'
				: 'text-indigo-700 bg-indigo-100 hover:bg-indigo-200 focus:ring-indigo-500') + " focus:outline-none focus:ring-2 focus:ring-offset-2");

				add_location(button1, file$7, 118, 6, 4256);
				attr_dev(div7, "class", "flex justify-between items-center");
				add_location(div7, file$7, 110, 4, 4031);
				attr_dev(div8, "class", "bg-gray-50 px-6 py-3");
				add_location(div8, file$7, 109, 2, 3992);
				attr_dev(div9, "class", "bg-white overflow-hidden shadow rounded-lg hover:shadow-md transition-shadow");
				add_location(div9, file$7, 54, 0, 1576);
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				insert_dev(target, div9, anchor);
				append_dev(div9, div6);
				append_dev(div6, div1);
				append_dev(div1, div0);
				append_dev(div0, h3);
				append_dev(h3, t0);
				append_dev(div0, t1);
				if (if_block0) if_block0.m(div0, null);
				append_dev(div1, t2);
				append_dev(div1, span0);
				append_dev(span0, t3);
				append_dev(div6, t4);
				append_dev(div6, p0);
				append_dev(p0, t5);
				append_dev(div6, t6);
				append_dev(div6, div2);
				append_dev(div2, h4);
				append_dev(div2, t8);
				append_dev(div2, p1);
				append_dev(p1, t9);
				append_dev(div6, t10);
				append_dev(div6, div4);
				append_dev(div4, div3);
				append_dev(div3, span1);
				append_dev(span1, svg0);
				append_dev(svg0, path0);
				append_dev(span1, t11);
				append_dev(span1, t12);
				append_dev(span1, t13);
				append_dev(div3, t14);
				append_dev(div3, span2);
				append_dev(span2, svg1);
				append_dev(svg1, path1);
				append_dev(span2, t15);
				append_dev(span2, t16);
				append_dev(span2, t17);
				append_dev(div4, t18);
				append_dev(div4, span3);
				append_dev(span3, t19);
				append_dev(span3, t20);
				append_dev(div6, t21);
				append_dev(div6, div5);
				append_dev(div5, t22);
				append_dev(div5, t23);
				append_dev(div5, t24);
				if (if_block1) if_block1.m(div5, null);
				append_dev(div9, t25);
				append_dev(div9, div8);
				append_dev(div8, div7);
				append_dev(div7, button0);
				append_dev(div7, t27);
				append_dev(div7, button1);
				if_block2.m(button1, null);

				if (!mounted) {
					dispose = [
						listen_dev(button0, "click", /*viewDetails*/ ctx[2], false, false, false, false),
						listen_dev(button1, "click", /*toggleSubscription*/ ctx[3], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, [dirty]) {
				if (dirty & /*list*/ 1 && t0_value !== (t0_value = /*list*/ ctx[0].name + "")) set_data_dev(t0, t0_value);

				if (/*isSubscribed*/ ctx[1]) {
					if (if_block0) ; else {
						if_block0 = create_if_block_2$4(ctx);
						if_block0.c();
						if_block0.m(div0, null);
					}
				} else if (if_block0) {
					if_block0.d(1);
					if_block0 = null;
				}

				if (dirty & /*list*/ 1 && t3_value !== (t3_value = /*list*/ ctx[0].update_cadence + "")) set_data_dev(t3, t3_value);

				if (dirty & /*list*/ 1 && span0_class_value !== (span0_class_value = "inline-flex items-center px-2 py-0.5 rounded text-xs font-medium " + getUpdateCadenceColor$1(/*list*/ ctx[0].update_cadence))) {
					attr_dev(span0, "class", span0_class_value);
				}

				if (dirty & /*list*/ 1 && t5_value !== (t5_value = /*list*/ ctx[0].description + "")) set_data_dev(t5, t5_value);
				if (dirty & /*list*/ 1 && t9_value !== (t9_value = /*list*/ ctx[0].criteria + "")) set_data_dev(t9, t9_value);
				if (dirty & /*list*/ 1 && t12_value !== (t12_value = (/*list*/ ctx[0].artist_count || 0) + "")) set_data_dev(t12, t12_value);
				if (dirty & /*list*/ 1 && t16_value !== (t16_value = (/*list*/ ctx[0].subscriber_count || 0) + "")) set_data_dev(t16, t16_value);
				if (dirty & /*list*/ 1 && t20_value !== (t20_value = /*list*/ ctx[0].version + "")) set_data_dev(t20, t20_value);
				if (dirty & /*list*/ 1 && t23_value !== (t23_value = formatDate$3(/*list*/ ctx[0].updated_at) + "")) set_data_dev(t23, t23_value);

				if (/*list*/ ctx[0].governance_url) {
					if (if_block1) {
						if_block1.p(ctx, dirty);
					} else {
						if_block1 = create_if_block_1$7(ctx);
						if_block1.c();
						if_block1.m(div5, null);
					}
				} else if (if_block1) {
					if_block1.d(1);
					if_block1 = null;
				}

				if (current_block_type !== (current_block_type = select_block_type(ctx))) {
					if_block2.d(1);
					if_block2 = current_block_type(ctx);

					if (if_block2) {
						if_block2.c();
						if_block2.m(button1, null);
					}
				}

				if (dirty & /*isSubscribed*/ 2 && button1_class_value !== (button1_class_value = "inline-flex items-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md " + (/*isSubscribed*/ ctx[1]
				? 'text-red-700 bg-red-100 hover:bg-red-200 focus:ring-red-500'
				: 'text-indigo-700 bg-indigo-100 hover:bg-indigo-200 focus:ring-indigo-500') + " focus:outline-none focus:ring-2 focus:ring-offset-2")) {
					attr_dev(button1, "class", button1_class_value);
				}
			},
			i: noop,
			o: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div9);
				}

				if (if_block0) if_block0.d();
				if (if_block1) if_block1.d();
				if_block2.d();
				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_fragment$7.name,
			type: "component",
			source: "",
			ctx
		});

		return block;
	}

	function formatDate$3(dateString) {
		return new Date(dateString).toLocaleDateString();
	}

	function getUpdateCadenceColor$1(cadence) {
		switch (cadence.toLowerCase()) {
			case 'daily':
				return 'text-red-600 bg-red-100';
			case 'weekly':
				return 'text-yellow-600 bg-yellow-100';
			case 'monthly':
				return 'text-green-600 bg-green-100';
			case 'as-needed':
				return 'text-blue-600 bg-blue-100';
			default:
				return 'text-gray-600 bg-gray-100';
		}
	}

	function instance$7($$self, $$props, $$invalidate) {
		let isSubscribed;
		let $subscribedListIds;
		validate_store(subscribedListIds, 'subscribedListIds');
		component_subscribe($$self, subscribedListIds, $$value => $$invalidate(4, $subscribedListIds = $$value));
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('CommunityListCard', slots, []);
		let { list } = $$props;

		async function viewDetails() {
			await communityActions.fetchListDetails(list.id);
		}

		async function toggleSubscription() {
			if (isSubscribed) {
				const result = await communityActions.unsubscribe(list.id);

				if (!result.success) {
					alert(`Failed to unsubscribe: ${result.message}`);
				}
			} else {
				// Show impact preview first
				const impact = await communityActions.getSubscriptionImpact(list.id);

				if (impact.success) {
					const confirmed = confirm(`This list will add ${impact.data.artists_to_add} artists to your DNP list. Continue?`);

					if (confirmed) {
						const result = await communityActions.subscribe(list.id);

						if (!result.success) {
							alert(`Failed to subscribe: ${result.message}`);
						}
					}
				}
			}
		}

		$$self.$$.on_mount.push(function () {
			if (list === undefined && !('list' in $$props || $$self.$$.bound[$$self.$$.props['list']])) {
				console.warn("<CommunityListCard> was created without expected prop 'list'");
			}
		});

		const writable_props = ['list'];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<CommunityListCard> was created with unknown prop '${key}'`);
		});

		$$self.$$set = $$props => {
			if ('list' in $$props) $$invalidate(0, list = $$props.list);
		};

		$$self.$capture_state = () => ({
			communityActions,
			subscribedListIds,
			list,
			viewDetails,
			toggleSubscription,
			formatDate: formatDate$3,
			getUpdateCadenceColor: getUpdateCadenceColor$1,
			isSubscribed,
			$subscribedListIds
		});

		$$self.$inject_state = $$props => {
			if ('list' in $$props) $$invalidate(0, list = $$props.list);
			if ('isSubscribed' in $$props) $$invalidate(1, isSubscribed = $$props.isSubscribed);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		$$self.$$.update = () => {
			if ($$self.$$.dirty & /*$subscribedListIds, list*/ 17) {
				$$invalidate(1, isSubscribed = $subscribedListIds.has(list.id));
			}
		};

		return [list, isSubscribed, viewDetails, toggleSubscription, $subscribedListIds];
	}

	class CommunityListCard extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance$7, create_fragment$7, safe_not_equal, { list: 0 });

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "CommunityListCard",
				options,
				id: create_fragment$7.name
			});
		}

		get list() {
			throw new Error("<CommunityListCard>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}

		set list(value) {
			throw new Error("<CommunityListCard>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
		}
	}

	/* src/lib/components/CommunityListDetail.svelte generated by Svelte v4.2.20 */

	const file$6 = "src/lib/components/CommunityListDetail.svelte";

	function get_each_context$2(ctx, list, i) {
		const child_ctx = ctx.slice();
		child_ctx[15] = list[i];
		return child_ctx;
	}

	function get_each_context_1(ctx, list, i) {
		const child_ctx = ctx.slice();
		child_ctx[18] = list[i];
		return child_ctx;
	}

	// (313:0) {:else}
	function create_else_block_3(ctx) {
		let div;
		let svg;
		let path;
		let t0;
		let h3;
		let t2;
		let p;

		const block = {
			c: function create() {
				div = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				h3 = element("h3");
				h3.textContent = "No list selected";
				t2 = space();
				p = element("p");
				p.textContent = "Select a list to view its details.";
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10");
				add_location(path, file$6, 324, 6, 13172);
				attr_dev(svg, "class", "mx-auto h-12 w-12 text-gray-400");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$6, 323, 4, 13066);
				attr_dev(h3, "class", "mt-2 text-sm font-medium text-gray-900");
				add_location(h3, file$6, 326, 4, 13415);
				attr_dev(p, "class", "mt-1 text-sm text-gray-500");
				add_location(p, file$6, 327, 4, 13492);
				attr_dev(div, "class", "text-center py-12");
				add_location(div, file$6, 322, 2, 13030);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, svg);
				append_dev(svg, path);
				append_dev(div, t0);
				append_dev(div, h3);
				append_dev(div, t2);
				append_dev(div, p);
			},
			p: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block_3.name,
			type: "else",
			source: "(313:0) {:else}",
			ctx
		});

		return block;
	}

	// (56:0) {#if list}
	function create_if_block$6(ctx) {
		let div23;
		let div15;
		let div1;
		let button0;
		let svg;
		let path;
		let t0;
		let t1;
		let div0;
		let t2;
		let span;
		let t3;
		let t4_value = /*list*/ ctx[0].version + "";
		let t4;
		let t5;
		let div14;
		let div12;
		let h1;
		let t6_value = /*list*/ ctx[0].name + "";
		let t6;
		let t7;
		let p0;
		let t8_value = /*list*/ ctx[0].description + "";
		let t8;
		let t9;
		let div11;
		let div4;
		let div2;
		let t10_value = (/*list*/ ctx[0].artists?.length || 0) + "";
		let t10;
		let t11;
		let div3;
		let t13;
		let div7;
		let div5;
		let t14_value = (/*list*/ ctx[0].subscriber_count || 0) + "";
		let t14;
		let t15;
		let div6;
		let t17;
		let div10;
		let div8;
		let t18_value = /*list*/ ctx[0].update_cadence + "";
		let t18;
		let t19;
		let div9;
		let t21;
		let div13;
		let button1;
		let button1_class_value;
		let t22;
		let t23;
		let div21;
		let h30;
		let t25;
		let div20;
		let div16;
		let h40;
		let t27;
		let p1;
		let t28_value = /*list*/ ctx[0].criteria + "";
		let t28;
		let t29;
		let div19;
		let div17;
		let h41;
		let t31;
		let p2;
		let t32_value = /*list*/ ctx[0].update_cadence + "";
		let t32;
		let t33;
		let div18;
		let h42;
		let t35;
		let p3;
		let t36_value = formatDate$2(/*list*/ ctx[0].updated_at) + "";
		let t36;
		let t37;
		let t38;
		let div22;
		let h31;
		let t39;
		let t40_value = (/*list*/ ctx[0].artists?.length || 0) + "";
		let t40;
		let t41;
		let t42;
		let mounted;
		let dispose;
		let if_block0 = /*isSubscribed*/ ctx[5] && create_if_block_9$1(ctx);

		function select_block_type_1(ctx, dirty) {
			if (/*isSubscribed*/ ctx[5]) return create_if_block_8$1;
			return create_else_block_2$2;
		}

		let current_block_type = select_block_type_1(ctx);
		let if_block1 = current_block_type(ctx);
		let if_block2 = /*showSubscriptionOptions*/ ctx[2] && create_if_block_7$3(ctx);
		let if_block3 = /*list*/ ctx[0].governance_url && create_if_block_6$3(ctx);

		function select_block_type_2(ctx, dirty) {
			if (/*$communityStore*/ ctx[1].isLoadingList) return create_if_block_1$6;
			if (/*list*/ ctx[0].artists && /*list*/ ctx[0].artists.length > 0) return create_if_block_2$3;
			return create_else_block_1$3;
		}

		let current_block_type_1 = select_block_type_2(ctx);
		let if_block4 = current_block_type_1(ctx);

		const block = {
			c: function create() {
				div23 = element("div");
				div15 = element("div");
				div1 = element("div");
				button0 = element("button");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = text("\n          Back to lists");
				t1 = space();
				div0 = element("div");
				if (if_block0) if_block0.c();
				t2 = space();
				span = element("span");
				t3 = text("v");
				t4 = text(t4_value);
				t5 = space();
				div14 = element("div");
				div12 = element("div");
				h1 = element("h1");
				t6 = text(t6_value);
				t7 = space();
				p0 = element("p");
				t8 = text(t8_value);
				t9 = space();
				div11 = element("div");
				div4 = element("div");
				div2 = element("div");
				t10 = text(t10_value);
				t11 = space();
				div3 = element("div");
				div3.textContent = "Artists";
				t13 = space();
				div7 = element("div");
				div5 = element("div");
				t14 = text(t14_value);
				t15 = space();
				div6 = element("div");
				div6.textContent = "Subscribers";
				t17 = space();
				div10 = element("div");
				div8 = element("div");
				t18 = text(t18_value);
				t19 = space();
				div9 = element("div");
				div9.textContent = "Updates";
				t21 = space();
				div13 = element("div");
				button1 = element("button");
				if_block1.c();
				t22 = space();
				if (if_block2) if_block2.c();
				t23 = space();
				div21 = element("div");
				h30 = element("h3");
				h30.textContent = "List Criteria & Governance";
				t25 = space();
				div20 = element("div");
				div16 = element("div");
				h40 = element("h4");
				h40.textContent = "Inclusion Criteria";
				t27 = space();
				p1 = element("p");
				t28 = text(t28_value);
				t29 = space();
				div19 = element("div");
				div17 = element("div");
				h41 = element("h4");
				h41.textContent = "Update Cadence";
				t31 = space();
				p2 = element("p");
				t32 = text(t32_value);
				t33 = space();
				div18 = element("div");
				h42 = element("h4");
				h42.textContent = "Last Updated";
				t35 = space();
				p3 = element("p");
				t36 = text(t36_value);
				t37 = space();
				if (if_block3) if_block3.c();
				t38 = space();
				div22 = element("div");
				h31 = element("h3");
				t39 = text("Artists (");
				t40 = text(t40_value);
				t41 = text(")");
				t42 = space();
				if_block4.c();
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M15 19l-7-7 7-7");
				add_location(path, file$6, 74, 12, 2306);
				attr_dev(svg, "class", "-ml-1 mr-2 h-5 w-5");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$6, 73, 10, 2207);
				attr_dev(button0, "class", "inline-flex items-center text-sm text-gray-500 hover:text-gray-700");
				add_location(button0, file$6, 69, 8, 2066);
				attr_dev(span, "class", "text-sm text-gray-500");
				add_location(span, file$6, 85, 10, 2744);
				attr_dev(div0, "class", "flex items-center space-x-2");
				add_location(div0, file$6, 79, 8, 2475);
				attr_dev(div1, "class", "flex items-center justify-between mb-4");
				add_location(div1, file$6, 68, 6, 2005);
				attr_dev(h1, "class", "text-2xl font-bold text-gray-900");
				add_location(h1, file$6, 91, 10, 2930);
				attr_dev(p0, "class", "mt-2 text-gray-600");
				add_location(p0, file$6, 92, 10, 3002);
				attr_dev(div2, "class", "text-lg font-semibold text-gray-900");
				add_location(div2, file$6, 96, 14, 3211);
				attr_dev(div3, "class", "text-sm text-gray-500");
				add_location(div3, file$6, 97, 14, 3308);
				attr_dev(div4, "class", "text-center p-3 bg-gray-50 rounded-lg");
				add_location(div4, file$6, 95, 12, 3145);
				attr_dev(div5, "class", "text-lg font-semibold text-gray-900");
				add_location(div5, file$6, 100, 14, 3454);
				attr_dev(div6, "class", "text-sm text-gray-500");
				add_location(div6, file$6, 101, 14, 3552);
				attr_dev(div7, "class", "text-center p-3 bg-gray-50 rounded-lg");
				add_location(div7, file$6, 99, 12, 3388);
				attr_dev(div8, "class", "text-lg font-semibold text-gray-900");
				add_location(div8, file$6, 104, 14, 3702);
				attr_dev(div9, "class", "text-sm text-gray-500");
				add_location(div9, file$6, 105, 14, 3793);
				attr_dev(div10, "class", "text-center p-3 bg-gray-50 rounded-lg");
				add_location(div10, file$6, 103, 12, 3636);
				attr_dev(div11, "class", "mt-4 grid grid-cols-1 gap-4 sm:grid-cols-3");
				add_location(div11, file$6, 94, 10, 3076);
				attr_dev(div12, "class", "flex-1");
				add_location(div12, file$6, 90, 8, 2899);

				attr_dev(button1, "class", button1_class_value = "inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm " + (/*isSubscribed*/ ctx[5]
				? 'text-red-700 bg-red-100 hover:bg-red-200 focus:ring-red-500'
				: 'text-white bg-indigo-600 hover:bg-indigo-700 focus:ring-indigo-500') + " focus:outline-none focus:ring-2 focus:ring-offset-2");

				add_location(button1, file$6, 111, 10, 3939);
				attr_dev(div13, "class", "ml-6");
				add_location(div13, file$6, 110, 8, 3910);
				attr_dev(div14, "class", "flex justify-between items-start");
				add_location(div14, file$6, 89, 6, 2844);
				attr_dev(div15, "class", "bg-white shadow rounded-lg p-6");
				add_location(div15, file$6, 67, 4, 1954);
				attr_dev(h30, "class", "text-lg font-medium text-gray-900 mb-4");
				add_location(h30, file$6, 208, 6, 8103);
				attr_dev(h40, "class", "text-sm font-medium text-gray-900");
				add_location(h40, file$6, 212, 10, 8247);
				attr_dev(p1, "class", "mt-1 text-sm text-gray-600");
				add_location(p1, file$6, 213, 10, 8327);
				add_location(div16, file$6, 211, 8, 8231);
				attr_dev(h41, "class", "text-sm font-medium text-gray-900");
				add_location(h41, file$6, 218, 12, 8497);
				attr_dev(p2, "class", "mt-1 text-sm text-gray-600 capitalize");
				add_location(p2, file$6, 219, 12, 8575);
				add_location(div17, file$6, 217, 10, 8479);
				attr_dev(h42, "class", "text-sm font-medium text-gray-900");
				add_location(h42, file$6, 222, 12, 8695);
				attr_dev(p3, "class", "mt-1 text-sm text-gray-600");
				add_location(p3, file$6, 223, 12, 8771);
				add_location(div18, file$6, 221, 10, 8677);
				attr_dev(div19, "class", "grid grid-cols-1 gap-4 sm:grid-cols-2");
				add_location(div19, file$6, 216, 8, 8417);
				attr_dev(div20, "class", "space-y-4");
				add_location(div20, file$6, 210, 6, 8199);
				attr_dev(div21, "class", "bg-white shadow rounded-lg p-6");
				add_location(div21, file$6, 207, 4, 8052);
				attr_dev(h31, "class", "text-lg font-medium text-gray-900 mb-4");
				add_location(h31, file$6, 244, 6, 9392);
				attr_dev(div22, "class", "bg-white shadow rounded-lg p-6");
				add_location(div22, file$6, 243, 4, 9341);
				attr_dev(div23, "class", "space-y-6");
				add_location(div23, file$6, 65, 2, 1906);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div23, anchor);
				append_dev(div23, div15);
				append_dev(div15, div1);
				append_dev(div1, button0);
				append_dev(button0, svg);
				append_dev(svg, path);
				append_dev(button0, t0);
				append_dev(div1, t1);
				append_dev(div1, div0);
				if (if_block0) if_block0.m(div0, null);
				append_dev(div0, t2);
				append_dev(div0, span);
				append_dev(span, t3);
				append_dev(span, t4);
				append_dev(div15, t5);
				append_dev(div15, div14);
				append_dev(div14, div12);
				append_dev(div12, h1);
				append_dev(h1, t6);
				append_dev(div12, t7);
				append_dev(div12, p0);
				append_dev(p0, t8);
				append_dev(div12, t9);
				append_dev(div12, div11);
				append_dev(div11, div4);
				append_dev(div4, div2);
				append_dev(div2, t10);
				append_dev(div4, t11);
				append_dev(div4, div3);
				append_dev(div11, t13);
				append_dev(div11, div7);
				append_dev(div7, div5);
				append_dev(div5, t14);
				append_dev(div7, t15);
				append_dev(div7, div6);
				append_dev(div11, t17);
				append_dev(div11, div10);
				append_dev(div10, div8);
				append_dev(div8, t18);
				append_dev(div10, t19);
				append_dev(div10, div9);
				append_dev(div14, t21);
				append_dev(div14, div13);
				append_dev(div13, button1);
				if_block1.m(button1, null);
				append_dev(div23, t22);
				if (if_block2) if_block2.m(div23, null);
				append_dev(div23, t23);
				append_dev(div23, div21);
				append_dev(div21, h30);
				append_dev(div21, t25);
				append_dev(div21, div20);
				append_dev(div20, div16);
				append_dev(div16, h40);
				append_dev(div16, t27);
				append_dev(div16, p1);
				append_dev(p1, t28);
				append_dev(div20, t29);
				append_dev(div20, div19);
				append_dev(div19, div17);
				append_dev(div17, h41);
				append_dev(div17, t31);
				append_dev(div17, p2);
				append_dev(p2, t32);
				append_dev(div19, t33);
				append_dev(div19, div18);
				append_dev(div18, h42);
				append_dev(div18, t35);
				append_dev(div18, p3);
				append_dev(p3, t36);
				append_dev(div20, t37);
				if (if_block3) if_block3.m(div20, null);
				append_dev(div23, t38);
				append_dev(div23, div22);
				append_dev(div22, h31);
				append_dev(h31, t39);
				append_dev(h31, t40);
				append_dev(h31, t41);
				append_dev(div22, t42);
				if_block4.m(div22, null);

				if (!mounted) {
					dispose = [
						listen_dev(button0, "click", /*goBack*/ ctx[6], false, false, false, false),
						listen_dev(button1, "click", /*toggleSubscription*/ ctx[7], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (/*isSubscribed*/ ctx[5]) {
					if (if_block0) ; else {
						if_block0 = create_if_block_9$1(ctx);
						if_block0.c();
						if_block0.m(div0, t2);
					}
				} else if (if_block0) {
					if_block0.d(1);
					if_block0 = null;
				}

				if (dirty & /*list*/ 1 && t4_value !== (t4_value = /*list*/ ctx[0].version + "")) set_data_dev(t4, t4_value);
				if (dirty & /*list*/ 1 && t6_value !== (t6_value = /*list*/ ctx[0].name + "")) set_data_dev(t6, t6_value);
				if (dirty & /*list*/ 1 && t8_value !== (t8_value = /*list*/ ctx[0].description + "")) set_data_dev(t8, t8_value);
				if (dirty & /*list*/ 1 && t10_value !== (t10_value = (/*list*/ ctx[0].artists?.length || 0) + "")) set_data_dev(t10, t10_value);
				if (dirty & /*list*/ 1 && t14_value !== (t14_value = (/*list*/ ctx[0].subscriber_count || 0) + "")) set_data_dev(t14, t14_value);
				if (dirty & /*list*/ 1 && t18_value !== (t18_value = /*list*/ ctx[0].update_cadence + "")) set_data_dev(t18, t18_value);

				if (current_block_type !== (current_block_type = select_block_type_1(ctx))) {
					if_block1.d(1);
					if_block1 = current_block_type(ctx);

					if (if_block1) {
						if_block1.c();
						if_block1.m(button1, null);
					}
				}

				if (dirty & /*isSubscribed*/ 32 && button1_class_value !== (button1_class_value = "inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm " + (/*isSubscribed*/ ctx[5]
				? 'text-red-700 bg-red-100 hover:bg-red-200 focus:ring-red-500'
				: 'text-white bg-indigo-600 hover:bg-indigo-700 focus:ring-indigo-500') + " focus:outline-none focus:ring-2 focus:ring-offset-2")) {
					attr_dev(button1, "class", button1_class_value);
				}

				if (/*showSubscriptionOptions*/ ctx[2]) {
					if (if_block2) {
						if_block2.p(ctx, dirty);
					} else {
						if_block2 = create_if_block_7$3(ctx);
						if_block2.c();
						if_block2.m(div23, t23);
					}
				} else if (if_block2) {
					if_block2.d(1);
					if_block2 = null;
				}

				if (dirty & /*list*/ 1 && t28_value !== (t28_value = /*list*/ ctx[0].criteria + "")) set_data_dev(t28, t28_value);
				if (dirty & /*list*/ 1 && t32_value !== (t32_value = /*list*/ ctx[0].update_cadence + "")) set_data_dev(t32, t32_value);
				if (dirty & /*list*/ 1 && t36_value !== (t36_value = formatDate$2(/*list*/ ctx[0].updated_at) + "")) set_data_dev(t36, t36_value);

				if (/*list*/ ctx[0].governance_url) {
					if (if_block3) {
						if_block3.p(ctx, dirty);
					} else {
						if_block3 = create_if_block_6$3(ctx);
						if_block3.c();
						if_block3.m(div20, null);
					}
				} else if (if_block3) {
					if_block3.d(1);
					if_block3 = null;
				}

				if (dirty & /*list*/ 1 && t40_value !== (t40_value = (/*list*/ ctx[0].artists?.length || 0) + "")) set_data_dev(t40, t40_value);

				if (current_block_type_1 === (current_block_type_1 = select_block_type_2(ctx)) && if_block4) {
					if_block4.p(ctx, dirty);
				} else {
					if_block4.d(1);
					if_block4 = current_block_type_1(ctx);

					if (if_block4) {
						if_block4.c();
						if_block4.m(div22, null);
					}
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div23);
				}

				if (if_block0) if_block0.d();
				if_block1.d();
				if (if_block2) if_block2.d();
				if (if_block3) if_block3.d();
				if_block4.d();
				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block$6.name,
			type: "if",
			source: "(56:0) {#if list}",
			ctx
		});

		return block;
	}

	// (72:10) {#if isSubscribed}
	function create_if_block_9$1(ctx) {
		let span;

		const block = {
			c: function create() {
				span = element("span");
				span.textContent = "Subscribed";
				attr_dev(span, "class", "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium text-green-800 bg-green-100");
				add_location(span, file$6, 81, 12, 2558);
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
			id: create_if_block_9$1.name,
			type: "if",
			source: "(72:10) {#if isSubscribed}",
			ctx
		});

		return block;
	}

	// (114:12) {:else}
	function create_else_block_2$2(ctx) {
		let svg;
		let path;
		let t;

		const block = {
			c: function create() {
				svg = svg_element("svg");
				path = svg_element("path");
				t = text("\n              Subscribe");
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M12 6v6m0 0v6m0-6h6m-6 0H6");
				add_location(path, file$6, 124, 16, 4785);
				attr_dev(svg, "class", "-ml-1 mr-2 h-5 w-5");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$6, 123, 14, 4682);
			},
			m: function mount(target, anchor) {
				insert_dev(target, svg, anchor);
				append_dev(svg, path);
				insert_dev(target, t, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(svg);
					detach_dev(t);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block_2$2.name,
			type: "else",
			source: "(114:12) {:else}",
			ctx
		});

		return block;
	}

	// (109:12) {#if isSubscribed}
	function create_if_block_8$1(ctx) {
		let svg;
		let path;
		let t;

		const block = {
			c: function create() {
				svg = svg_element("svg");
				path = svg_element("path");
				t = text("\n              Unsubscribe");
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M6 18L18 6M6 6l12 12");
				add_location(path, file$6, 119, 16, 4503);
				attr_dev(svg, "class", "-ml-1 mr-2 h-5 w-5");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$6, 118, 14, 4400);
			},
			m: function mount(target, anchor) {
				insert_dev(target, svg, anchor);
				append_dev(svg, path);
				insert_dev(target, t, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(svg);
					detach_dev(t);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_8$1.name,
			type: "if",
			source: "(109:12) {#if isSubscribed}",
			ctx
		});

		return block;
	}

	// (126:4) {#if showSubscriptionOptions}
	function create_if_block_7$3(ctx) {
		let div9;
		let h3;
		let t1;
		let div7;
		let div3;
		let h4;
		let t3;
		let div2;
		let div0;
		let input0;
		let t4;
		let label0;
		let t6;
		let div1;
		let input1;
		let input1_value_value;
		let value_has_changed = false;
		let t7;
		let label1;
		let t8;
		let t9_value = /*list*/ ctx[0].version + "";
		let t9;
		let t10;
		let t11;
		let div6;
		let div4;
		let input2;
		let t12;
		let div5;
		let label2;
		let t14;
		let p;
		let t16;
		let div8;
		let button0;
		let t18;
		let button1;
		let binding_group;
		let mounted;
		let dispose;
		binding_group = init_binding_group(/*$$binding_groups*/ ctx[11][0]);

		const block = {
			c: function create() {
				div9 = element("div");
				h3 = element("h3");
				h3.textContent = "Subscription Options";
				t1 = space();
				div7 = element("div");
				div3 = element("div");
				h4 = element("h4");
				h4.textContent = "Version Pinning";
				t3 = space();
				div2 = element("div");
				div0 = element("div");
				input0 = element("input");
				t4 = space();
				label0 = element("label");
				label0.textContent = "Auto-update to latest version (recommended)";
				t6 = space();
				div1 = element("div");
				input1 = element("input");
				t7 = space();
				label1 = element("label");
				t8 = text("Pin to current version (v");
				t9 = text(t9_value);
				t10 = text(")");
				t11 = space();
				div6 = element("div");
				div4 = element("div");
				input2 = element("input");
				t12 = space();
				div5 = element("div");
				label2 = element("label");
				label2.textContent = "Enable automatic updates";
				t14 = space();
				p = element("p");
				p.textContent = "Receive notifications when the list is updated and apply changes automatically.";
				t16 = space();
				div8 = element("div");
				button0 = element("button");
				button0.textContent = "Cancel";
				t18 = space();
				button1 = element("button");
				button1.textContent = "Subscribe";
				attr_dev(h3, "class", "text-lg font-medium text-gray-900 mb-4");
				add_location(h3, file$6, 136, 8, 5172);
				attr_dev(h4, "class", "block text-sm font-medium text-gray-700");
				add_location(h4, file$6, 140, 12, 5318);
				attr_dev(input0, "id", "auto-update");
				attr_dev(input0, "type", "radio");
				input0.__value = null;
				set_input_value(input0, input0.__value);
				attr_dev(input0, "class", "focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300");
				add_location(input0, file$6, 143, 16, 5494);
				attr_dev(label0, "for", "auto-update");
				attr_dev(label0, "class", "ml-3 block text-sm text-gray-700");
				add_location(label0, file$6, 150, 16, 5766);
				attr_dev(div0, "class", "flex items-center");
				add_location(div0, file$6, 142, 14, 5446);
				attr_dev(input1, "id", "pin-version");
				attr_dev(input1, "type", "radio");
				input1.__value = input1_value_value = /*list*/ ctx[0].version;
				set_input_value(input1, input1.__value);
				attr_dev(input1, "class", "focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300");
				add_location(input1, file$6, 155, 16, 6003);
				attr_dev(label1, "for", "pin-version");
				attr_dev(label1, "class", "ml-3 block text-sm text-gray-700");
				add_location(label1, file$6, 162, 16, 6283);
				attr_dev(div1, "class", "flex items-center");
				add_location(div1, file$6, 154, 14, 5955);
				attr_dev(div2, "class", "mt-2 space-y-2");
				add_location(div2, file$6, 141, 12, 5403);
				add_location(div3, file$6, 139, 10, 5300);
				attr_dev(input2, "id", "auto-update-checkbox");
				attr_dev(input2, "type", "checkbox");
				attr_dev(input2, "class", "focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300 rounded");
				add_location(input2, file$6, 171, 14, 6605);
				attr_dev(div4, "class", "flex items-center h-5");
				add_location(div4, file$6, 170, 12, 6555);
				attr_dev(label2, "for", "auto-update-checkbox");
				attr_dev(label2, "class", "font-medium text-gray-700");
				add_location(label2, file$6, 179, 14, 6911);
				attr_dev(p, "class", "text-gray-500");
				add_location(p, file$6, 182, 14, 7058);
				attr_dev(div5, "class", "ml-3 text-sm");
				add_location(div5, file$6, 178, 12, 6870);
				attr_dev(div6, "class", "flex items-start");
				add_location(div6, file$6, 169, 10, 6512);
				attr_dev(div7, "class", "space-y-4");
				add_location(div7, file$6, 138, 8, 5266);
				attr_dev(button0, "class", "px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500");
				add_location(button0, file$6, 190, 10, 7323);
				attr_dev(button1, "class", "px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500");
				add_location(button1, file$6, 196, 10, 7651);
				attr_dev(div8, "class", "mt-6 flex justify-end space-x-3");
				add_location(div8, file$6, 189, 8, 7267);
				attr_dev(div9, "class", "bg-white shadow rounded-lg p-6 border-2 border-indigo-200");
				add_location(div9, file$6, 135, 6, 5092);
				binding_group.p(input0, input1);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div9, anchor);
				append_dev(div9, h3);
				append_dev(div9, t1);
				append_dev(div9, div7);
				append_dev(div7, div3);
				append_dev(div3, h4);
				append_dev(div3, t3);
				append_dev(div3, div2);
				append_dev(div2, div0);
				append_dev(div0, input0);
				input0.checked = input0.__value === /*versionPinned*/ ctx[3];
				append_dev(div0, t4);
				append_dev(div0, label0);
				append_dev(div2, t6);
				append_dev(div2, div1);
				append_dev(div1, input1);
				input1.checked = input1.__value === /*versionPinned*/ ctx[3];
				append_dev(div1, t7);
				append_dev(div1, label1);
				append_dev(label1, t8);
				append_dev(label1, t9);
				append_dev(label1, t10);
				append_dev(div7, t11);
				append_dev(div7, div6);
				append_dev(div6, div4);
				append_dev(div4, input2);
				input2.checked = /*autoUpdate*/ ctx[4];
				append_dev(div6, t12);
				append_dev(div6, div5);
				append_dev(div5, label2);
				append_dev(div5, t14);
				append_dev(div5, p);
				append_dev(div9, t16);
				append_dev(div9, div8);
				append_dev(div8, button0);
				append_dev(div8, t18);
				append_dev(div8, button1);

				if (!mounted) {
					dispose = [
						listen_dev(input0, "change", /*input0_change_handler*/ ctx[10]),
						listen_dev(input1, "change", /*input1_change_handler*/ ctx[12]),
						listen_dev(input2, "change", /*input2_change_handler*/ ctx[13]),
						listen_dev(button0, "click", /*click_handler*/ ctx[14], false, false, false, false),
						listen_dev(button1, "click", /*confirmSubscription*/ ctx[8], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (dirty & /*versionPinned*/ 8) {
					input0.checked = input0.__value === /*versionPinned*/ ctx[3];
				}

				if (dirty & /*list*/ 1 && input1_value_value !== (input1_value_value = /*list*/ ctx[0].version)) {
					prop_dev(input1, "__value", input1_value_value);
					set_input_value(input1, input1.__value);
					value_has_changed = true;
				}

				if (value_has_changed || dirty & /*versionPinned*/ 8) {
					input1.checked = input1.__value === /*versionPinned*/ ctx[3];
				}

				if (dirty & /*list*/ 1 && t9_value !== (t9_value = /*list*/ ctx[0].version + "")) set_data_dev(t9, t9_value);

				if (dirty & /*autoUpdate*/ 16) {
					input2.checked = /*autoUpdate*/ ctx[4];
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div9);
				}

				binding_group.r();
				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_7$3.name,
			type: "if",
			source: "(126:4) {#if showSubscriptionOptions}",
			ctx
		});

		return block;
	}

	// (219:8) {#if list.governance_url}
	function create_if_block_6$3(ctx) {
		let div;
		let h4;
		let t1;
		let a;
		let t2;
		let a_href_value;

		const block = {
			c: function create() {
				div = element("div");
				h4 = element("h4");
				h4.textContent = "Governance Process";
				t1 = space();
				a = element("a");
				t2 = text("View governance documentation →");
				attr_dev(h4, "class", "text-sm font-medium text-gray-900");
				add_location(h4, file$6, 229, 12, 8946);
				attr_dev(a, "href", a_href_value = /*list*/ ctx[0].governance_url);
				attr_dev(a, "target", "_blank");
				attr_dev(a, "class", "mt-1 text-sm text-indigo-600 hover:text-indigo-500");
				add_location(a, file$6, 230, 12, 9028);
				add_location(div, file$6, 228, 10, 8928);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, h4);
				append_dev(div, t1);
				append_dev(div, a);
				append_dev(a, t2);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*list*/ 1 && a_href_value !== (a_href_value = /*list*/ ctx[0].governance_url)) {
					attr_dev(a, "href", a_href_value);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_6$3.name,
			type: "if",
			source: "(219:8) {#if list.governance_url}",
			ctx
		});

		return block;
	}

	// (303:6) {:else}
	function create_else_block_1$3(ctx) {
		let div;
		let svg;
		let path;
		let t0;
		let p;

		const block = {
			c: function create() {
				div = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				p = element("p");
				p.textContent = "No artists in this list yet.";
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z");
				add_location(path, file$6, 314, 12, 12730);
				attr_dev(svg, "class", "mx-auto h-8 w-8 text-gray-400");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$6, 313, 10, 12620);
				attr_dev(p, "class", "mt-2 text-sm text-gray-500");
				add_location(p, file$6, 316, 10, 12902);
				attr_dev(div, "class", "text-center py-6");
				add_location(div, file$6, 312, 8, 12579);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, svg);
				append_dev(svg, path);
				append_dev(div, t0);
				append_dev(div, p);
			},
			p: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block_1$3.name,
			type: "else",
			source: "(303:6) {:else}",
			ctx
		});

		return block;
	}

	// (248:56) 
	function create_if_block_2$3(ctx) {
		let div;
		let each_value = ensure_array_like_dev(/*list*/ ctx[0].artists);
		let each_blocks = [];

		for (let i = 0; i < each_value.length; i += 1) {
			each_blocks[i] = create_each_block$2(get_each_context$2(ctx, each_value, i));
		}

		const block = {
			c: function create() {
				div = element("div");

				for (let i = 0; i < each_blocks.length; i += 1) {
					each_blocks[i].c();
				}

				attr_dev(div, "class", "space-y-3 max-h-96 overflow-y-auto");
				add_location(div, file$6, 257, 8, 10149);
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
				if (dirty & /*list, formatDate, getProviderBadges*/ 1) {
					each_value = ensure_array_like_dev(/*list*/ ctx[0].artists);
					let i;

					for (i = 0; i < each_value.length; i += 1) {
						const child_ctx = get_each_context$2(ctx, each_value, i);

						if (each_blocks[i]) {
							each_blocks[i].p(child_ctx, dirty);
						} else {
							each_blocks[i] = create_each_block$2(child_ctx);
							each_blocks[i].c();
							each_blocks[i].m(div, null);
						}
					}

					for (; i < each_blocks.length; i += 1) {
						each_blocks[i].d(1);
					}

					each_blocks.length = each_value.length;
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
			id: create_if_block_2$3.name,
			type: "if",
			source: "(248:56) ",
			ctx
		});

		return block;
	}

	// (240:6) {#if $communityStore.isLoadingList}
	function create_if_block_1$6(ctx) {
		let div;
		let svg;
		let circle;
		let path;
		let t0;
		let p;

		const block = {
			c: function create() {
				div = element("div");
				svg = svg_element("svg");
				circle = svg_element("circle");
				path = svg_element("path");
				t0 = space();
				p = element("p");
				p.textContent = "Loading artists...";
				attr_dev(circle, "class", "opacity-25");
				attr_dev(circle, "cx", "12");
				attr_dev(circle, "cy", "12");
				attr_dev(circle, "r", "10");
				attr_dev(circle, "stroke", "currentColor");
				attr_dev(circle, "stroke-width", "4");
				add_location(circle, file$6, 251, 12, 9701);
				attr_dev(path, "class", "opacity-75");
				attr_dev(path, "fill", "currentColor");
				attr_dev(path, "d", "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z");
				add_location(path, file$6, 252, 12, 9812);
				attr_dev(svg, "class", "animate-spin mx-auto h-6 w-6 text-gray-400");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				add_location(svg, file$6, 250, 10, 9600);
				attr_dev(p, "class", "mt-2 text-sm text-gray-500");
				add_location(p, file$6, 254, 10, 10008);
				attr_dev(div, "class", "text-center py-6");
				add_location(div, file$6, 249, 8, 9559);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, svg);
				append_dev(svg, circle);
				append_dev(svg, path);
				append_dev(div, t0);
				append_dev(div, p);
			},
			p: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_1$6.name,
			type: "if",
			source: "(240:6) {#if $communityStore.isLoadingList}",
			ctx
		});

		return block;
	}

	// (259:16) {:else}
	function create_else_block$6(ctx) {
		let div;
		let svg;
		let path;

		const block = {
			c: function create() {
				div = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z");
				add_location(path, file$6, 270, 22, 10904);
				attr_dev(svg, "class", "h-5 w-5 text-gray-600");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$6, 269, 20, 10792);
				attr_dev(div, "class", "h-10 w-10 rounded-full bg-gray-300 flex items-center justify-center");
				add_location(div, file$6, 268, 18, 10690);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, svg);
				append_dev(svg, path);
			},
			p: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block$6.name,
			type: "else",
			source: "(259:16) {:else}",
			ctx
		});

		return block;
	}

	// (253:16) {#if item.artist.metadata.image}
	function create_if_block_5$3(ctx) {
		let img;
		let img_src_value;
		let img_alt_value;

		const block = {
			c: function create() {
				img = element("img");
				if (!src_url_equal(img.src, img_src_value = /*item*/ ctx[15].artist.metadata.image)) attr_dev(img, "src", img_src_value);
				attr_dev(img, "alt", img_alt_value = /*item*/ ctx[15].artist.canonical_name);
				attr_dev(img, "class", "h-10 w-10 rounded-full object-cover");
				add_location(img, file$6, 262, 18, 10452);
			},
			m: function mount(target, anchor) {
				insert_dev(target, img, anchor);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*list*/ 1 && !src_url_equal(img.src, img_src_value = /*item*/ ctx[15].artist.metadata.image)) {
					attr_dev(img, "src", img_src_value);
				}

				if (dirty & /*list*/ 1 && img_alt_value !== (img_alt_value = /*item*/ ctx[15].artist.canonical_name)) {
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
			id: create_if_block_5$3.name,
			type: "if",
			source: "(253:16) {#if item.artist.metadata.image}",
			ctx
		});

		return block;
	}

	// (271:18) {#if item.artist.metadata.genres && item.artist.metadata.genres.length > 0}
	function create_if_block_4$3(ctx) {
		let div;
		let t_value = /*item*/ ctx[15].artist.metadata.genres.slice(0, 2).join(', ') + "";
		let t;

		const block = {
			c: function create() {
				div = element("div");
				t = text(t_value);
				attr_dev(div, "class", "text-xs text-gray-500");
				add_location(div, file$6, 280, 20, 11416);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, t);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*list*/ 1 && t_value !== (t_value = /*item*/ ctx[15].artist.metadata.genres.slice(0, 2).join(', ') + "")) set_data_dev(t, t_value);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_4$3.name,
			type: "if",
			source: "(271:18) {#if item.artist.metadata.genres && item.artist.metadata.genres.length > 0}",
			ctx
		});

		return block;
	}

	// (277:20) {#each getProviderBadges(item.artist) as badge}
	function create_each_block_1(ctx) {
		let span;
		let t0_value = /*badge*/ ctx[18].name + "";
		let t0;
		let t1;
		let span_class_value;

		const block = {
			c: function create() {
				span = element("span");
				t0 = text(t0_value);
				t1 = space();
				attr_dev(span, "class", span_class_value = "inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium " + /*badge*/ ctx[18].color);
				add_location(span, file$6, 286, 22, 11720);
			},
			m: function mount(target, anchor) {
				insert_dev(target, span, anchor);
				append_dev(span, t0);
				append_dev(span, t1);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*list*/ 1 && t0_value !== (t0_value = /*badge*/ ctx[18].name + "")) set_data_dev(t0, t0_value);

				if (dirty & /*list*/ 1 && span_class_value !== (span_class_value = "inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium " + /*badge*/ ctx[18].color)) {
					attr_dev(span, "class", span_class_value);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(span);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_each_block_1.name,
			type: "each",
			source: "(277:20) {#each getProviderBadges(item.artist) as badge}",
			ctx
		});

		return block;
	}

	// (290:16) {#if item.rationale_link}
	function create_if_block_3$3(ctx) {
		let a;
		let t;
		let a_href_value;

		const block = {
			c: function create() {
				a = element("a");
				t = text("View rationale");
				attr_dev(a, "href", a_href_value = /*item*/ ctx[15].rationale_link);
				attr_dev(a, "target", "_blank");
				attr_dev(a, "class", "text-xs text-indigo-600 hover:text-indigo-500");
				add_location(a, file$6, 299, 18, 12221);
			},
			m: function mount(target, anchor) {
				insert_dev(target, a, anchor);
				append_dev(a, t);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*list*/ 1 && a_href_value !== (a_href_value = /*item*/ ctx[15].rationale_link)) {
					attr_dev(a, "href", a_href_value);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(a);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_3$3.name,
			type: "if",
			source: "(290:16) {#if item.rationale_link}",
			ctx
		});

		return block;
	}

	// (250:10) {#each list.artists as item}
	function create_each_block$2(ctx) {
		let div6;
		let div3;
		let t0;
		let div2;
		let div0;
		let t1_value = /*item*/ ctx[15].artist.canonical_name + "";
		let t1;
		let t2;
		let t3;
		let div1;
		let t4;
		let div5;
		let div4;
		let t5;
		let t6_value = formatDate$2(/*item*/ ctx[15].added_at) + "";
		let t6;
		let t7;
		let t8;

		function select_block_type_3(ctx, dirty) {
			if (/*item*/ ctx[15].artist.metadata.image) return create_if_block_5$3;
			return create_else_block$6;
		}

		let current_block_type = select_block_type_3(ctx);
		let if_block0 = current_block_type(ctx);
		let if_block1 = /*item*/ ctx[15].artist.metadata.genres && /*item*/ ctx[15].artist.metadata.genres.length > 0 && create_if_block_4$3(ctx);
		let each_value_1 = ensure_array_like_dev(getProviderBadges(/*item*/ ctx[15].artist));
		let each_blocks = [];

		for (let i = 0; i < each_value_1.length; i += 1) {
			each_blocks[i] = create_each_block_1(get_each_context_1(ctx, each_value_1, i));
		}

		let if_block2 = /*item*/ ctx[15].rationale_link && create_if_block_3$3(ctx);

		const block = {
			c: function create() {
				div6 = element("div");
				div3 = element("div");
				if_block0.c();
				t0 = space();
				div2 = element("div");
				div0 = element("div");
				t1 = text(t1_value);
				t2 = space();
				if (if_block1) if_block1.c();
				t3 = space();
				div1 = element("div");

				for (let i = 0; i < each_blocks.length; i += 1) {
					each_blocks[i].c();
				}

				t4 = space();
				div5 = element("div");
				div4 = element("div");
				t5 = text("Added ");
				t6 = text(t6_value);
				t7 = space();
				if (if_block2) if_block2.c();
				t8 = space();
				attr_dev(div0, "class", "text-sm font-medium text-gray-900");
				add_location(div0, file$6, 276, 18, 11180);
				attr_dev(div1, "class", "flex space-x-1 mt-1");
				add_location(div1, file$6, 284, 18, 11596);
				add_location(div2, file$6, 275, 16, 11156);
				attr_dev(div3, "class", "flex items-center space-x-3");
				add_location(div3, file$6, 260, 14, 10343);
				attr_dev(div4, "class", "text-xs text-gray-500");
				add_location(div4, file$6, 295, 16, 12050);
				attr_dev(div5, "class", "text-right");
				add_location(div5, file$6, 294, 14, 12009);
				attr_dev(div6, "class", "flex items-center justify-between py-3 px-4 bg-gray-50 rounded-lg");
				add_location(div6, file$6, 259, 12, 10249);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div6, anchor);
				append_dev(div6, div3);
				if_block0.m(div3, null);
				append_dev(div3, t0);
				append_dev(div3, div2);
				append_dev(div2, div0);
				append_dev(div0, t1);
				append_dev(div2, t2);
				if (if_block1) if_block1.m(div2, null);
				append_dev(div2, t3);
				append_dev(div2, div1);

				for (let i = 0; i < each_blocks.length; i += 1) {
					if (each_blocks[i]) {
						each_blocks[i].m(div1, null);
					}
				}

				append_dev(div6, t4);
				append_dev(div6, div5);
				append_dev(div5, div4);
				append_dev(div4, t5);
				append_dev(div4, t6);
				append_dev(div5, t7);
				if (if_block2) if_block2.m(div5, null);
				append_dev(div6, t8);
			},
			p: function update(ctx, dirty) {
				if (current_block_type === (current_block_type = select_block_type_3(ctx)) && if_block0) {
					if_block0.p(ctx, dirty);
				} else {
					if_block0.d(1);
					if_block0 = current_block_type(ctx);

					if (if_block0) {
						if_block0.c();
						if_block0.m(div3, t0);
					}
				}

				if (dirty & /*list*/ 1 && t1_value !== (t1_value = /*item*/ ctx[15].artist.canonical_name + "")) set_data_dev(t1, t1_value);

				if (/*item*/ ctx[15].artist.metadata.genres && /*item*/ ctx[15].artist.metadata.genres.length > 0) {
					if (if_block1) {
						if_block1.p(ctx, dirty);
					} else {
						if_block1 = create_if_block_4$3(ctx);
						if_block1.c();
						if_block1.m(div2, t3);
					}
				} else if (if_block1) {
					if_block1.d(1);
					if_block1 = null;
				}

				if (dirty & /*getProviderBadges, list*/ 1) {
					each_value_1 = ensure_array_like_dev(getProviderBadges(/*item*/ ctx[15].artist));
					let i;

					for (i = 0; i < each_value_1.length; i += 1) {
						const child_ctx = get_each_context_1(ctx, each_value_1, i);

						if (each_blocks[i]) {
							each_blocks[i].p(child_ctx, dirty);
						} else {
							each_blocks[i] = create_each_block_1(child_ctx);
							each_blocks[i].c();
							each_blocks[i].m(div1, null);
						}
					}

					for (; i < each_blocks.length; i += 1) {
						each_blocks[i].d(1);
					}

					each_blocks.length = each_value_1.length;
				}

				if (dirty & /*list*/ 1 && t6_value !== (t6_value = formatDate$2(/*item*/ ctx[15].added_at) + "")) set_data_dev(t6, t6_value);

				if (/*item*/ ctx[15].rationale_link) {
					if (if_block2) {
						if_block2.p(ctx, dirty);
					} else {
						if_block2 = create_if_block_3$3(ctx);
						if_block2.c();
						if_block2.m(div5, null);
					}
				} else if (if_block2) {
					if_block2.d(1);
					if_block2 = null;
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div6);
				}

				if_block0.d();
				if (if_block1) if_block1.d();
				destroy_each(each_blocks, detaching);
				if (if_block2) if_block2.d();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_each_block$2.name,
			type: "each",
			source: "(250:10) {#each list.artists as item}",
			ctx
		});

		return block;
	}

	function create_fragment$6(ctx) {
		let if_block_anchor;

		function select_block_type(ctx, dirty) {
			if (/*list*/ ctx[0]) return create_if_block$6;
			return create_else_block_3;
		}

		let current_block_type = select_block_type(ctx);
		let if_block = current_block_type(ctx);

		const block = {
			c: function create() {
				if_block.c();
				if_block_anchor = empty();
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				if_block.m(target, anchor);
				insert_dev(target, if_block_anchor, anchor);
			},
			p: function update(ctx, [dirty]) {
				if (current_block_type === (current_block_type = select_block_type(ctx)) && if_block) {
					if_block.p(ctx, dirty);
				} else {
					if_block.d(1);
					if_block = current_block_type(ctx);

					if (if_block) {
						if_block.c();
						if_block.m(if_block_anchor.parentNode, if_block_anchor);
					}
				}
			},
			i: noop,
			o: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(if_block_anchor);
				}

				if_block.d(detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_fragment$6.name,
			type: "component",
			source: "",
			ctx
		});

		return block;
	}

	function formatDate$2(dateString) {
		return new Date(dateString).toLocaleDateString();
	}

	function getProviderBadges(artist) {
		const badges = [];

		if (artist.external_ids.spotify) badges.push({
			name: 'Spotify',
			color: 'bg-green-100 text-green-800'
		});

		if (artist.external_ids.apple) badges.push({
			name: 'Apple',
			color: 'bg-gray-100 text-gray-800'
		});

		if (artist.external_ids.musicbrainz) badges.push({
			name: 'MusicBrainz',
			color: 'bg-blue-100 text-blue-800'
		});

		return badges;
	}

	function instance$6($$self, $$props, $$invalidate) {
		let list;
		let isSubscribed;
		let $subscribedListIds;
		let $communityStore;
		validate_store(subscribedListIds, 'subscribedListIds');
		component_subscribe($$self, subscribedListIds, $$value => $$invalidate(9, $subscribedListIds = $$value));
		validate_store(communityStore, 'communityStore');
		component_subscribe($$self, communityStore, $$value => $$invalidate(1, $communityStore = $$value));
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('CommunityListDetail', slots, []);
		let showSubscriptionOptions = false;
		let versionPinned = null;
		let autoUpdate = true;

		function goBack() {
			communityActions.clearCurrentList();
		}

		async function toggleSubscription() {
			if (!list) return;

			if (isSubscribed) {
				const result = await communityActions.unsubscribe(list.id);

				if (!result.success) {
					alert(`Failed to unsubscribe: ${result.message}`);
				}
			} else {
				$$invalidate(2, showSubscriptionOptions = true);
			}
		}

		async function confirmSubscription() {
			if (!list) return;

			// Get impact preview
			const impact = await communityActions.getSubscriptionImpact(list.id);

			if (impact.success) {
				const confirmed = confirm(`This list will add ${impact.data.artists_to_add} artists to your DNP list. Continue?`);

				if (confirmed) {
					const result = await communityActions.subscribe(list.id, versionPinned || undefined, autoUpdate);

					if (result.success) {
						$$invalidate(2, showSubscriptionOptions = false);
					} else {
						alert(`Failed to subscribe: ${result.message}`);
					}
				}
			}
		}

		const writable_props = [];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<CommunityListDetail> was created with unknown prop '${key}'`);
		});

		const $$binding_groups = [[]];

		function input0_change_handler() {
			versionPinned = this.__value;
			$$invalidate(3, versionPinned);
		}

		function input1_change_handler() {
			versionPinned = this.__value;
			$$invalidate(3, versionPinned);
		}

		function input2_change_handler() {
			autoUpdate = this.checked;
			$$invalidate(4, autoUpdate);
		}

		const click_handler = () => $$invalidate(2, showSubscriptionOptions = false);

		$$self.$capture_state = () => ({
			communityActions,
			communityStore,
			subscribedListIds,
			showSubscriptionOptions,
			versionPinned,
			autoUpdate,
			goBack,
			toggleSubscription,
			confirmSubscription,
			formatDate: formatDate$2,
			getProviderBadges,
			list,
			isSubscribed,
			$subscribedListIds,
			$communityStore
		});

		$$self.$inject_state = $$props => {
			if ('showSubscriptionOptions' in $$props) $$invalidate(2, showSubscriptionOptions = $$props.showSubscriptionOptions);
			if ('versionPinned' in $$props) $$invalidate(3, versionPinned = $$props.versionPinned);
			if ('autoUpdate' in $$props) $$invalidate(4, autoUpdate = $$props.autoUpdate);
			if ('list' in $$props) $$invalidate(0, list = $$props.list);
			if ('isSubscribed' in $$props) $$invalidate(5, isSubscribed = $$props.isSubscribed);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		$$self.$$.update = () => {
			if ($$self.$$.dirty & /*$communityStore*/ 2) {
				$$invalidate(0, list = $communityStore.currentList);
			}

			if ($$self.$$.dirty & /*list, $subscribedListIds*/ 513) {
				$$invalidate(5, isSubscribed = list ? $subscribedListIds.has(list.id) : false);
			}
		};

		return [
			list,
			$communityStore,
			showSubscriptionOptions,
			versionPinned,
			autoUpdate,
			isSubscribed,
			goBack,
			toggleSubscription,
			confirmSubscription,
			$subscribedListIds,
			input0_change_handler,
			$$binding_groups,
			input1_change_handler,
			input2_change_handler,
			click_handler
		];
	}

	class CommunityListDetail extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance$6, create_fragment$6, safe_not_equal, {});

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "CommunityListDetail",
				options,
				id: create_fragment$6.name
			});
		}
	}

	/* src/lib/components/CreateCommunityList.svelte generated by Svelte v4.2.20 */
	const file$5 = "src/lib/components/CreateCommunityList.svelte";

	// (158:2) {#if error}
	function create_if_block_1$5(ctx) {
		let div;
		let t;

		const block = {
			c: function create() {
				div = element("div");
				t = text(/*error*/ ctx[7]);
				attr_dev(div, "class", "text-red-600 text-sm");
				add_location(div, file$5, 165, 4, 5183);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, t);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*error*/ 128) set_data_dev(t, /*error*/ ctx[7]);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_1$5.name,
			type: "if",
			source: "(158:2) {#if error}",
			ctx
		});

		return block;
	}

	// (184:6) {:else}
	function create_else_block$5(ctx) {
		let t;

		const block = {
			c: function create() {
				t = text("Create List");
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
			id: create_else_block$5.name,
			type: "else",
			source: "(184:6) {:else}",
			ctx
		});

		return block;
	}

	// (178:6) {#if isCreating}
	function create_if_block$5(ctx) {
		let svg;
		let circle;
		let path;
		let t;

		const block = {
			c: function create() {
				svg = svg_element("svg");
				circle = svg_element("circle");
				path = svg_element("path");
				t = text("\n        Creating...");
				attr_dev(circle, "class", "opacity-25");
				attr_dev(circle, "cx", "12");
				attr_dev(circle, "cy", "12");
				attr_dev(circle, "r", "10");
				attr_dev(circle, "stroke", "currentColor");
				attr_dev(circle, "stroke-width", "4");
				add_location(circle, file$5, 186, 10, 6172);
				attr_dev(path, "class", "opacity-75");
				attr_dev(path, "fill", "currentColor");
				attr_dev(path, "d", "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z");
				add_location(path, file$5, 187, 10, 6281);
				attr_dev(svg, "class", "animate-spin -ml-1 mr-2 h-4 w-4 text-white");
				attr_dev(svg, "xmlns", "http://www.w3.org/2000/svg");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				add_location(svg, file$5, 185, 8, 6038);
			},
			m: function mount(target, anchor) {
				insert_dev(target, svg, anchor);
				append_dev(svg, circle);
				append_dev(svg, path);
				insert_dev(target, t, anchor);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(svg);
					detach_dev(t);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block$5.name,
			type: "if",
			source: "(178:6) {#if isCreating}",
			ctx
		});

		return block;
	}

	function create_fragment$5(ctx) {
		let form;
		let div0;
		let label0;
		let t1;
		let input0;
		let t2;
		let div1;
		let label1;
		let t4;
		let textarea0;
		let t5;
		let div2;
		let label2;
		let t7;
		let textarea1;
		let t8;
		let p0;
		let t10;
		let div3;
		let label3;
		let t12;
		let input1;
		let t13;
		let p1;
		let t15;
		let div4;
		let label4;
		let t17;
		let select;
		let option0;
		let option1;
		let option2;
		let option3;
		let t22;
		let div8;
		let h4;
		let t24;
		let div7;
		let div5;
		let input2;
		let t25;
		let label5;
		let t27;
		let div6;
		let input3;
		let t28;
		let label6;
		let t30;
		let t31;
		let div9;
		let button0;
		let t33;
		let button1;
		let button1_disabled_value;
		let t34;
		let div14;
		let div13;
		let div10;
		let svg;
		let path;
		let t35;
		let div12;
		let h3;
		let t37;
		let div11;
		let ul;
		let li0;
		let t39;
		let li1;
		let t41;
		let li2;
		let t43;
		let li3;
		let t45;
		let li4;
		let binding_group;
		let mounted;
		let dispose;
		let if_block0 = /*error*/ ctx[7] && create_if_block_1$5(ctx);

		function select_block_type(ctx, dirty) {
			if (/*isCreating*/ ctx[6]) return create_if_block$5;
			return create_else_block$5;
		}

		let current_block_type = select_block_type(ctx);
		let if_block1 = current_block_type(ctx);
		binding_group = init_binding_group(/*$$binding_groups*/ ctx[16][0]);

		const block = {
			c: function create() {
				form = element("form");
				div0 = element("div");
				label0 = element("label");
				label0.textContent = "List Name *";
				t1 = space();
				input0 = element("input");
				t2 = space();
				div1 = element("div");
				label1 = element("label");
				label1.textContent = "Description *";
				t4 = space();
				textarea0 = element("textarea");
				t5 = space();
				div2 = element("div");
				label2 = element("label");
				label2.textContent = "Inclusion Criteria *";
				t7 = space();
				textarea1 = element("textarea");
				t8 = space();
				p0 = element("p");
				p0.textContent = "Criteria must be factual and neutral. Avoid subjective terms or personal opinions.";
				t10 = space();
				div3 = element("div");
				label3 = element("label");
				label3.textContent = "Governance Documentation URL";
				t12 = space();
				input1 = element("input");
				t13 = space();
				p1 = element("p");
				p1.textContent = "Link to documentation explaining your list's governance process and appeals procedure.";
				t15 = space();
				div4 = element("div");
				label4 = element("label");
				label4.textContent = "Update Cadence";
				t17 = space();
				select = element("select");
				option0 = element("option");
				option0.textContent = "Daily";
				option1 = element("option");
				option1.textContent = "Weekly";
				option2 = element("option");
				option2.textContent = "Monthly";
				option3 = element("option");
				option3.textContent = "As Needed";
				t22 = space();
				div8 = element("div");
				h4 = element("h4");
				h4.textContent = "Visibility";
				t24 = space();
				div7 = element("div");
				div5 = element("div");
				input2 = element("input");
				t25 = space();
				label5 = element("label");
				label5.textContent = "Public - Anyone can discover and subscribe";
				t27 = space();
				div6 = element("div");
				input3 = element("input");
				t28 = space();
				label6 = element("label");
				label6.textContent = "Private - Only you can manage, others need direct link";
				t30 = space();
				if (if_block0) if_block0.c();
				t31 = space();
				div9 = element("div");
				button0 = element("button");
				button0.textContent = "Cancel";
				t33 = space();
				button1 = element("button");
				if_block1.c();
				t34 = space();
				div14 = element("div");
				div13 = element("div");
				div10 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t35 = space();
				div12 = element("div");
				h3 = element("h3");
				h3.textContent = "Community List Guidelines";
				t37 = space();
				div11 = element("div");
				ul = element("ul");
				li0 = element("li");
				li0.textContent = "Use neutral, factual language in criteria";
				t39 = space();
				li1 = element("li");
				li1.textContent = "Provide clear governance and appeals processes";
				t41 = space();
				li2 = element("li");
				li2.textContent = "Maintain transparency about list updates";
				t43 = space();
				li3 = element("li");
				li3.textContent = "Respect platform terms of service";
				t45 = space();
				li4 = element("li");
				li4.textContent = "Focus on user preferences, not personal judgments";
				attr_dev(label0, "for", "name");
				attr_dev(label0, "class", "block text-sm font-medium text-gray-700");
				add_location(label0, file$5, 53, 4, 1317);
				attr_dev(input0, "id", "name");
				attr_dev(input0, "type", "text");
				attr_dev(input0, "placeholder", "e.g., Controversial Artists List");
				attr_dev(input0, "class", "mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 placeholder-gray-500 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm");
				input0.required = true;
				add_location(input0, file$5, 56, 4, 1419);
				add_location(div0, file$5, 52, 2, 1307);
				attr_dev(label1, "for", "description");
				attr_dev(label1, "class", "block text-sm font-medium text-gray-700");
				add_location(label1, file$5, 68, 4, 1777);
				attr_dev(textarea0, "id", "description");
				attr_dev(textarea0, "rows", "3");
				attr_dev(textarea0, "placeholder", "Describe the purpose and scope of this list...");
				attr_dev(textarea0, "class", "mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 placeholder-gray-500 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm");
				textarea0.required = true;
				add_location(textarea0, file$5, 71, 4, 1888);
				add_location(div1, file$5, 67, 2, 1767);
				attr_dev(label2, "for", "criteria");
				attr_dev(label2, "class", "block text-sm font-medium text-gray-700");
				add_location(label2, file$5, 83, 4, 2281);
				attr_dev(textarea1, "id", "criteria");
				attr_dev(textarea1, "rows", "4");
				attr_dev(textarea1, "placeholder", "Define clear, neutral criteria for including artists in this list. Avoid subjective language or personal opinions.");
				attr_dev(textarea1, "class", "mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 placeholder-gray-500 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm");
				textarea1.required = true;
				add_location(textarea1, file$5, 86, 4, 2396);
				attr_dev(p0, "class", "mt-1 text-xs text-gray-500");
				add_location(p0, file$5, 94, 4, 2813);
				add_location(div2, file$5, 82, 2, 2271);
				attr_dev(label3, "for", "governance-url");
				attr_dev(label3, "class", "block text-sm font-medium text-gray-700");
				add_location(label3, file$5, 101, 4, 2998);
				attr_dev(input1, "id", "governance-url");
				attr_dev(input1, "type", "url");
				attr_dev(input1, "placeholder", "https://example.com/governance-process");
				attr_dev(input1, "class", "mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 placeholder-gray-500 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm");
				add_location(input1, file$5, 104, 4, 3127);
				attr_dev(p1, "class", "mt-1 text-xs text-gray-500");
				add_location(p1, file$5, 111, 4, 3453);
				add_location(div3, file$5, 100, 2, 2988);
				attr_dev(label4, "for", "update-cadence");
				attr_dev(label4, "class", "block text-sm font-medium text-gray-700");
				add_location(label4, file$5, 118, 4, 3642);
				option0.__value = "daily";
				set_input_value(option0, option0.__value);
				add_location(option0, file$5, 126, 6, 3988);
				option1.__value = "weekly";
				set_input_value(option1, option1.__value);
				add_location(option1, file$5, 127, 6, 4031);
				option2.__value = "monthly";
				set_input_value(option2, option2.__value);
				add_location(option2, file$5, 128, 6, 4076);
				option3.__value = "as-needed";
				set_input_value(option3, option3.__value);
				add_location(option3, file$5, 129, 6, 4123);
				attr_dev(select, "id", "update-cadence");
				attr_dev(select, "class", "mt-1 block w-full border border-gray-300 rounded-md px-3 py-2 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm");
				if (/*updateCadence*/ ctx[4] === void 0) add_render_callback(() => /*select_change_handler*/ ctx[14].call(select));
				add_location(select, file$5, 121, 4, 3757);
				add_location(div4, file$5, 117, 2, 3632);
				attr_dev(h4, "class", "block text-sm font-medium text-gray-700");
				add_location(h4, file$5, 135, 4, 4226);
				attr_dev(input2, "id", "public");
				attr_dev(input2, "type", "radio");
				input2.__value = "public";
				set_input_value(input2, input2.__value);
				attr_dev(input2, "class", "focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300");
				add_location(input2, file$5, 138, 8, 4373);
				attr_dev(label5, "for", "public");
				attr_dev(label5, "class", "ml-3 block text-sm text-gray-700");
				add_location(label5, file$5, 145, 8, 4583);
				attr_dev(div5, "class", "flex items-center");
				add_location(div5, file$5, 137, 6, 4333);
				attr_dev(input3, "id", "private");
				attr_dev(input3, "type", "radio");
				input3.__value = "private";
				set_input_value(input3, input3.__value);
				attr_dev(input3, "class", "focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300");
				add_location(input3, file$5, 150, 8, 4774);
				attr_dev(label6, "for", "private");
				attr_dev(label6, "class", "ml-3 block text-sm text-gray-700");
				add_location(label6, file$5, 157, 8, 4986);
				attr_dev(div6, "class", "flex items-center");
				add_location(div6, file$5, 149, 6, 4734);
				attr_dev(div7, "class", "mt-2 space-y-2");
				add_location(div7, file$5, 136, 4, 4298);
				add_location(div8, file$5, 134, 2, 4216);
				attr_dev(button0, "type", "button");
				attr_dev(button0, "class", "px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500");
				add_location(button0, file$5, 172, 4, 5324);
				attr_dev(button1, "type", "submit");
				button1.disabled = button1_disabled_value = /*isCreating*/ ctx[6] || !/*name*/ ctx[0].trim() || !/*description*/ ctx[1].trim() || !/*criteria*/ ctx[2].trim();
				attr_dev(button1, "class", "px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed");
				add_location(button1, file$5, 179, 4, 5628);
				attr_dev(div9, "class", "flex justify-end space-x-3");
				add_location(div9, file$5, 171, 2, 5279);
				attr_dev(path, "fill-rule", "evenodd");
				attr_dev(path, "d", "M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z");
				attr_dev(path, "clip-rule", "evenodd");
				add_location(path, file$5, 201, 10, 6799);
				attr_dev(svg, "class", "h-5 w-5 text-yellow-400");
				attr_dev(svg, "viewBox", "0 0 20 20");
				attr_dev(svg, "fill", "currentColor");
				add_location(svg, file$5, 200, 8, 6711);
				attr_dev(div10, "class", "flex-shrink-0");
				add_location(div10, file$5, 199, 6, 6675);
				attr_dev(h3, "class", "text-sm font-medium text-yellow-800");
				add_location(h3, file$5, 205, 8, 7123);
				add_location(li0, file$5, 210, 12, 7340);
				add_location(li1, file$5, 211, 12, 7403);
				add_location(li2, file$5, 212, 12, 7471);
				add_location(li3, file$5, 213, 12, 7533);
				add_location(li4, file$5, 214, 12, 7588);
				attr_dev(ul, "class", "list-disc list-inside space-y-1");
				add_location(ul, file$5, 209, 10, 7283);
				attr_dev(div11, "class", "mt-2 text-sm text-yellow-700");
				add_location(div11, file$5, 208, 8, 7230);
				attr_dev(div12, "class", "ml-3");
				add_location(div12, file$5, 204, 6, 7096);
				attr_dev(div13, "class", "flex");
				add_location(div13, file$5, 198, 4, 6650);
				attr_dev(div14, "class", "bg-yellow-50 border border-yellow-200 rounded-md p-4");
				add_location(div14, file$5, 197, 2, 6579);
				attr_dev(form, "class", "space-y-6");
				add_location(form, file$5, 50, 0, 1224);
				binding_group.p(input2, input3);
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				insert_dev(target, form, anchor);
				append_dev(form, div0);
				append_dev(div0, label0);
				append_dev(div0, t1);
				append_dev(div0, input0);
				set_input_value(input0, /*name*/ ctx[0]);
				append_dev(form, t2);
				append_dev(form, div1);
				append_dev(div1, label1);
				append_dev(div1, t4);
				append_dev(div1, textarea0);
				set_input_value(textarea0, /*description*/ ctx[1]);
				append_dev(form, t5);
				append_dev(form, div2);
				append_dev(div2, label2);
				append_dev(div2, t7);
				append_dev(div2, textarea1);
				set_input_value(textarea1, /*criteria*/ ctx[2]);
				append_dev(div2, t8);
				append_dev(div2, p0);
				append_dev(form, t10);
				append_dev(form, div3);
				append_dev(div3, label3);
				append_dev(div3, t12);
				append_dev(div3, input1);
				set_input_value(input1, /*governanceUrl*/ ctx[3]);
				append_dev(div3, t13);
				append_dev(div3, p1);
				append_dev(form, t15);
				append_dev(form, div4);
				append_dev(div4, label4);
				append_dev(div4, t17);
				append_dev(div4, select);
				append_dev(select, option0);
				append_dev(select, option1);
				append_dev(select, option2);
				append_dev(select, option3);
				select_option(select, /*updateCadence*/ ctx[4], true);
				append_dev(form, t22);
				append_dev(form, div8);
				append_dev(div8, h4);
				append_dev(div8, t24);
				append_dev(div8, div7);
				append_dev(div7, div5);
				append_dev(div5, input2);
				input2.checked = input2.__value === /*visibility*/ ctx[5];
				append_dev(div5, t25);
				append_dev(div5, label5);
				append_dev(div7, t27);
				append_dev(div7, div6);
				append_dev(div6, input3);
				input3.checked = input3.__value === /*visibility*/ ctx[5];
				append_dev(div6, t28);
				append_dev(div6, label6);
				append_dev(form, t30);
				if (if_block0) if_block0.m(form, null);
				append_dev(form, t31);
				append_dev(form, div9);
				append_dev(div9, button0);
				append_dev(div9, t33);
				append_dev(div9, button1);
				if_block1.m(button1, null);
				append_dev(form, t34);
				append_dev(form, div14);
				append_dev(div14, div13);
				append_dev(div13, div10);
				append_dev(div10, svg);
				append_dev(svg, path);
				append_dev(div13, t35);
				append_dev(div13, div12);
				append_dev(div12, h3);
				append_dev(div12, t37);
				append_dev(div12, div11);
				append_dev(div11, ul);
				append_dev(ul, li0);
				append_dev(ul, t39);
				append_dev(ul, li1);
				append_dev(ul, t41);
				append_dev(ul, li2);
				append_dev(ul, t43);
				append_dev(ul, li3);
				append_dev(ul, t45);
				append_dev(ul, li4);

				if (!mounted) {
					dispose = [
						listen_dev(input0, "input", /*input0_input_handler*/ ctx[10]),
						listen_dev(textarea0, "input", /*textarea0_input_handler*/ ctx[11]),
						listen_dev(textarea1, "input", /*textarea1_input_handler*/ ctx[12]),
						listen_dev(input1, "input", /*input1_input_handler*/ ctx[13]),
						listen_dev(select, "change", /*select_change_handler*/ ctx[14]),
						listen_dev(input2, "change", /*input2_change_handler*/ ctx[15]),
						listen_dev(input3, "change", /*input3_change_handler*/ ctx[17]),
						listen_dev(button0, "click", /*click_handler*/ ctx[18], false, false, false, false),
						listen_dev(form, "submit", prevent_default(/*handleSubmit*/ ctx[9]), false, true, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, [dirty]) {
				if (dirty & /*name*/ 1 && input0.value !== /*name*/ ctx[0]) {
					set_input_value(input0, /*name*/ ctx[0]);
				}

				if (dirty & /*description*/ 2) {
					set_input_value(textarea0, /*description*/ ctx[1]);
				}

				if (dirty & /*criteria*/ 4) {
					set_input_value(textarea1, /*criteria*/ ctx[2]);
				}

				if (dirty & /*governanceUrl*/ 8 && input1.value !== /*governanceUrl*/ ctx[3]) {
					set_input_value(input1, /*governanceUrl*/ ctx[3]);
				}

				if (dirty & /*updateCadence*/ 16) {
					select_option(select, /*updateCadence*/ ctx[4]);
				}

				if (dirty & /*visibility*/ 32) {
					input2.checked = input2.__value === /*visibility*/ ctx[5];
				}

				if (dirty & /*visibility*/ 32) {
					input3.checked = input3.__value === /*visibility*/ ctx[5];
				}

				if (/*error*/ ctx[7]) {
					if (if_block0) {
						if_block0.p(ctx, dirty);
					} else {
						if_block0 = create_if_block_1$5(ctx);
						if_block0.c();
						if_block0.m(form, t31);
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
						if_block1.m(button1, null);
					}
				}

				if (dirty & /*isCreating, name, description, criteria*/ 71 && button1_disabled_value !== (button1_disabled_value = /*isCreating*/ ctx[6] || !/*name*/ ctx[0].trim() || !/*description*/ ctx[1].trim() || !/*criteria*/ ctx[2].trim())) {
					prop_dev(button1, "disabled", button1_disabled_value);
				}
			},
			i: noop,
			o: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(form);
				}

				if (if_block0) if_block0.d();
				if_block1.d();
				binding_group.r();
				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_fragment$5.name,
			type: "component",
			source: "",
			ctx
		});

		return block;
	}

	function instance$5($$self, $$props, $$invalidate) {
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('CreateCommunityList', slots, []);
		const dispatch = createEventDispatcher();
		let name = '';
		let description = '';
		let criteria = '';
		let governanceUrl = '';
		let updateCadence = 'monthly';
		let visibility = 'public';
		let isCreating = false;
		let error = '';

		async function handleSubmit() {
			if (!name.trim() || !description.trim() || !criteria.trim()) {
				$$invalidate(7, error = 'Please fill in all required fields');
				return;
			}

			$$invalidate(6, isCreating = true);
			$$invalidate(7, error = '');

			const result = await communityActions.createList({
				name: name.trim(),
				description: description.trim(),
				criteria: criteria.trim(),
				governance_url: governanceUrl.trim() || undefined,
				update_cadence: updateCadence,
				visibility
			});

			if (result.success) {
				// Reset form
				$$invalidate(0, name = '');

				$$invalidate(1, description = '');
				$$invalidate(2, criteria = '');
				$$invalidate(3, governanceUrl = '');
				$$invalidate(4, updateCadence = 'monthly');
				$$invalidate(5, visibility = 'public');
				dispatch('listCreated');
			} else {
				$$invalidate(7, error = result.message || 'Failed to create community list');
			}

			$$invalidate(6, isCreating = false);
		}

		const writable_props = [];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<CreateCommunityList> was created with unknown prop '${key}'`);
		});

		const $$binding_groups = [[]];

		function input0_input_handler() {
			name = this.value;
			$$invalidate(0, name);
		}

		function textarea0_input_handler() {
			description = this.value;
			$$invalidate(1, description);
		}

		function textarea1_input_handler() {
			criteria = this.value;
			$$invalidate(2, criteria);
		}

		function input1_input_handler() {
			governanceUrl = this.value;
			$$invalidate(3, governanceUrl);
		}

		function select_change_handler() {
			updateCadence = select_value(this);
			$$invalidate(4, updateCadence);
		}

		function input2_change_handler() {
			visibility = this.__value;
			$$invalidate(5, visibility);
		}

		function input3_change_handler() {
			visibility = this.__value;
			$$invalidate(5, visibility);
		}

		const click_handler = () => dispatch('listCreated');

		$$self.$capture_state = () => ({
			createEventDispatcher,
			communityActions,
			dispatch,
			name,
			description,
			criteria,
			governanceUrl,
			updateCadence,
			visibility,
			isCreating,
			error,
			handleSubmit
		});

		$$self.$inject_state = $$props => {
			if ('name' in $$props) $$invalidate(0, name = $$props.name);
			if ('description' in $$props) $$invalidate(1, description = $$props.description);
			if ('criteria' in $$props) $$invalidate(2, criteria = $$props.criteria);
			if ('governanceUrl' in $$props) $$invalidate(3, governanceUrl = $$props.governanceUrl);
			if ('updateCadence' in $$props) $$invalidate(4, updateCadence = $$props.updateCadence);
			if ('visibility' in $$props) $$invalidate(5, visibility = $$props.visibility);
			if ('isCreating' in $$props) $$invalidate(6, isCreating = $$props.isCreating);
			if ('error' in $$props) $$invalidate(7, error = $$props.error);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		return [
			name,
			description,
			criteria,
			governanceUrl,
			updateCadence,
			visibility,
			isCreating,
			error,
			dispatch,
			handleSubmit,
			input0_input_handler,
			textarea0_input_handler,
			textarea1_input_handler,
			input1_input_handler,
			select_change_handler,
			input2_change_handler,
			$$binding_groups,
			input3_change_handler,
			click_handler
		];
	}

	class CreateCommunityList extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance$5, create_fragment$5, safe_not_equal, {});

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "CreateCommunityList",
				options,
				id: create_fragment$5.name
			});
		}
	}

	/* src/lib/components/MySubscriptions.svelte generated by Svelte v4.2.20 */
	const file$4 = "src/lib/components/MySubscriptions.svelte";

	function get_each_context$1(ctx, list, i) {
		const child_ctx = ctx.slice();
		child_ctx[11] = list[i];
		return child_ctx;
	}

	// (63:2) {:else}
	function create_else_block$4(ctx) {
		let div;
		let ul;
		let each_value = ensure_array_like_dev(/*subscriptions*/ ctx[0]);
		let each_blocks = [];

		for (let i = 0; i < each_value.length; i += 1) {
			each_blocks[i] = create_each_block$1(get_each_context$1(ctx, each_value, i));
		}

		const block = {
			c: function create() {
				div = element("div");
				ul = element("ul");

				for (let i = 0; i < each_blocks.length; i += 1) {
					each_blocks[i].c();
				}

				attr_dev(ul, "class", "divide-y divide-gray-200");
				add_location(ul, file$4, 73, 6, 2568);
				attr_dev(div, "class", "bg-white shadow overflow-hidden sm:rounded-md");
				add_location(div, file$4, 72, 4, 2502);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, ul);

				for (let i = 0; i < each_blocks.length; i += 1) {
					if (each_blocks[i]) {
						each_blocks[i].m(ul, null);
					}
				}
			},
			p: function update(ctx, dirty) {
				if (dirty & /*subscriptions, handleAutoUpdateChange, updateSubscription, undefined, unsubscribe, viewListDetails, formatDate, getUpdateCadenceColor*/ 31) {
					each_value = ensure_array_like_dev(/*subscriptions*/ ctx[0]);
					let i;

					for (i = 0; i < each_value.length; i += 1) {
						const child_ctx = get_each_context$1(ctx, each_value, i);

						if (each_blocks[i]) {
							each_blocks[i].p(child_ctx, dirty);
						} else {
							each_blocks[i] = create_each_block$1(child_ctx);
							each_blocks[i].c();
							each_blocks[i].m(ul, null);
						}
					}

					for (; i < each_blocks.length; i += 1) {
						each_blocks[i].d(1);
					}

					each_blocks.length = each_value.length;
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
			id: create_else_block$4.name,
			type: "else",
			source: "(63:2) {:else}",
			ctx
		});

		return block;
	}

	// (52:2) {#if subscriptions.length === 0}
	function create_if_block$4(ctx) {
		let div;
		let svg;
		let path;
		let t0;
		let h3;
		let t2;
		let p;

		const block = {
			c: function create() {
				div = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				h3 = element("h3");
				h3.textContent = "No subscriptions yet";
				t2 = space();
				p = element("p");
				p.textContent = "Browse community lists to find ones that match your preferences.";
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10");
				add_location(path, file$4, 63, 8, 1992);
				attr_dev(svg, "class", "mx-auto h-12 w-12 text-gray-400");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$4, 62, 6, 1884);
				attr_dev(h3, "class", "mt-2 text-sm font-medium text-gray-900");
				add_location(h3, file$4, 65, 6, 2239);
				attr_dev(p, "class", "mt-1 text-sm text-gray-500");
				add_location(p, file$4, 66, 6, 2322);
				attr_dev(div, "class", "text-center py-12");
				add_location(div, file$4, 61, 4, 1846);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, svg);
				append_dev(svg, path);
				append_dev(div, t0);
				append_dev(div, h3);
				append_dev(div, t2);
				append_dev(div, p);
			},
			p: noop,
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
			source: "(52:2) {#if subscriptions.length === 0}",
			ctx
		});

		return block;
	}

	// (92:24) {#if subscription.version_pinned}
	function create_if_block_1$4(ctx) {
		let t0;
		let t1_value = /*subscription*/ ctx[11].version_pinned + "";
		let t1;
		let t2;

		const block = {
			c: function create() {
				t0 = text("(pinned to v");
				t1 = text(t1_value);
				t2 = text(")");
			},
			m: function mount(target, anchor) {
				insert_dev(target, t0, anchor);
				insert_dev(target, t1, anchor);
				insert_dev(target, t2, anchor);
			},
			p: function update(ctx, dirty) {
				if (dirty & /*subscriptions*/ 1 && t1_value !== (t1_value = /*subscription*/ ctx[11].version_pinned + "")) set_data_dev(t1, t1_value);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(t0);
					detach_dev(t1);
					detach_dev(t2);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_1$4.name,
			type: "if",
			source: "(92:24) {#if subscription.version_pinned}",
			ctx
		});

		return block;
	}

	// (67:8) {#each subscriptions as subscription}
	function create_each_block$1(ctx) {
		let li;
		let div18;
		let div7;
		let div5;
		let div1;
		let div0;
		let svg;
		let path;
		let t0;
		let div4;
		let div2;
		let p0;
		let t1_value = /*subscription*/ ctx[11].list.name + "";
		let t1;
		let t2;
		let span;
		let t3_value = /*subscription*/ ctx[11].list.update_cadence + "";
		let t3;
		let span_class_value;
		let t4;
		let div3;
		let p1;
		let t5;
		let t6_value = formatDate$1(/*subscription*/ ctx[11].created_at) + "";
		let t6;
		let t7;
		let t8_value = /*subscription*/ ctx[11].list.version + "";
		let t8;
		let t9;
		let t10;
		let p2;
		let t11_value = /*subscription*/ ctx[11].list.description + "";
		let t11;
		let t12;
		let div6;
		let button0;
		let t14;
		let button1;
		let t16;
		let div17;
		let div16;
		let div11;
		let h50;
		let t18;
		let div10;
		let div8;
		let input0;
		let input0_id_value;
		let input0_checked_value;
		let t19;
		let label0;
		let t20;
		let label0_for_value;
		let t21;
		let div9;
		let input1;
		let input1_id_value;
		let input1_checked_value;
		let t22;
		let label1;
		let t23;
		let t24_value = /*subscription*/ ctx[11].list.version + "";
		let t24;
		let label1_for_value;
		let t25;
		let div15;
		let h51;
		let t27;
		let div14;
		let div12;
		let input2;
		let input2_id_value;
		let input2_checked_value;
		let t28;
		let div13;
		let label2;
		let t29;
		let label2_for_value;
		let t30;
		let p3;
		let t32;
		let mounted;
		let dispose;
		let if_block = /*subscription*/ ctx[11].version_pinned && create_if_block_1$4(ctx);

		function click_handler() {
			return /*click_handler*/ ctx[6](/*subscription*/ ctx[11]);
		}

		function click_handler_1() {
			return /*click_handler_1*/ ctx[7](/*subscription*/ ctx[11]);
		}

		function change_handler() {
			return /*change_handler*/ ctx[8](/*subscription*/ ctx[11]);
		}

		function change_handler_1() {
			return /*change_handler_1*/ ctx[9](/*subscription*/ ctx[11]);
		}

		function change_handler_2(...args) {
			return /*change_handler_2*/ ctx[10](/*subscription*/ ctx[11], ...args);
		}

		const block = {
			c: function create() {
				li = element("li");
				div18 = element("div");
				div7 = element("div");
				div5 = element("div");
				div1 = element("div");
				div0 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				div4 = element("div");
				div2 = element("div");
				p0 = element("p");
				t1 = text(t1_value);
				t2 = space();
				span = element("span");
				t3 = text(t3_value);
				t4 = space();
				div3 = element("div");
				p1 = element("p");
				t5 = text("Subscribed ");
				t6 = text(t6_value);
				t7 = text("\n                        • v");
				t8 = text(t8_value);
				t9 = space();
				if (if_block) if_block.c();
				t10 = space();
				p2 = element("p");
				t11 = text(t11_value);
				t12 = space();
				div6 = element("div");
				button0 = element("button");
				button0.textContent = "View Details";
				t14 = space();
				button1 = element("button");
				button1.textContent = "Unsubscribe";
				t16 = space();
				div17 = element("div");
				div16 = element("div");
				div11 = element("div");
				h50 = element("h5");
				h50.textContent = "Version Preference";
				t18 = space();
				div10 = element("div");
				div8 = element("div");
				input0 = element("input");
				t19 = space();
				label0 = element("label");
				t20 = text("Auto-update to latest");
				t21 = space();
				div9 = element("div");
				input1 = element("input");
				t22 = space();
				label1 = element("label");
				t23 = text("Pin to v");
				t24 = text(t24_value);
				t25 = space();
				div15 = element("div");
				h51 = element("h5");
				h51.textContent = "Update Notifications";
				t27 = space();
				div14 = element("div");
				div12 = element("div");
				input2 = element("input");
				t28 = space();
				div13 = element("div");
				label2 = element("label");
				t29 = text("Enable automatic updates");
				t30 = space();
				p3 = element("p");
				p3.textContent = "Apply changes when the list is updated";
				t32 = space();
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10");
				add_location(path, file$4, 82, 24, 3109);
				attr_dev(svg, "class", "h-5 w-5 text-indigo-600");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$4, 81, 22, 2993);
				attr_dev(div0, "class", "h-10 w-10 rounded-full bg-indigo-100 flex items-center justify-center");
				add_location(div0, file$4, 80, 20, 2887);
				attr_dev(div1, "class", "flex-shrink-0");
				add_location(div1, file$4, 79, 18, 2839);
				attr_dev(p0, "class", "text-sm font-medium text-gray-900");
				add_location(p0, file$4, 88, 22, 3529);
				attr_dev(span, "class", span_class_value = "ml-2 inline-flex items-center px-2 py-0.5 rounded text-xs font-medium " + getUpdateCadenceColor(/*subscription*/ ctx[11].list.update_cadence));
				add_location(span, file$4, 91, 22, 3673);
				attr_dev(div2, "class", "flex items-center");
				add_location(div2, file$4, 87, 20, 3475);
				add_location(p1, file$4, 96, 22, 4033);
				attr_dev(div3, "class", "mt-1 flex items-center text-sm text-gray-500");
				add_location(div3, file$4, 95, 20, 3952);
				attr_dev(p2, "class", "mt-1 text-sm text-gray-600 line-clamp-1");
				add_location(p2, file$4, 104, 20, 4396);
				attr_dev(div4, "class", "ml-4");
				add_location(div4, file$4, 86, 18, 3436);
				attr_dev(div5, "class", "flex items-center");
				add_location(div5, file$4, 78, 16, 2789);
				attr_dev(button0, "class", "text-indigo-600 hover:text-indigo-900 text-sm font-medium");
				add_location(button0, file$4, 111, 18, 4668);
				attr_dev(button1, "class", "text-red-600 hover:text-red-900 text-sm font-medium");
				add_location(button1, file$4, 117, 18, 4936);
				attr_dev(div6, "class", "flex items-center space-x-2");
				add_location(div6, file$4, 110, 16, 4608);
				attr_dev(div7, "class", "flex items-center justify-between");
				add_location(div7, file$4, 77, 14, 2725);
				attr_dev(h50, "class", "block text-xs font-medium text-gray-700 mb-2");
				add_location(h50, file$4, 131, 20, 5521);
				attr_dev(input0, "id", input0_id_value = "auto-" + /*subscription*/ ctx[11].list_id);
				attr_dev(input0, "type", "radio");
				input0.checked = input0_checked_value = !/*subscription*/ ctx[11].version_pinned;
				attr_dev(input0, "class", "focus:ring-indigo-500 h-3 w-3 text-indigo-600 border-gray-300");
				add_location(input0, file$4, 134, 24, 5724);
				attr_dev(label0, "for", label0_for_value = "auto-" + /*subscription*/ ctx[11].list_id);
				attr_dev(label0, "class", "ml-2 block text-xs text-gray-700");
				add_location(label0, file$4, 141, 24, 6163);
				attr_dev(div8, "class", "flex items-center");
				add_location(div8, file$4, 133, 22, 5668);
				attr_dev(input1, "id", input1_id_value = "pin-" + /*subscription*/ ctx[11].list_id);
				attr_dev(input1, "type", "radio");
				input1.checked = input1_checked_value = !!/*subscription*/ ctx[11].version_pinned;
				attr_dev(input1, "class", "focus:ring-indigo-500 h-3 w-3 text-indigo-600 border-gray-300");
				add_location(input1, file$4, 146, 24, 6434);
				attr_dev(label1, "for", label1_for_value = "pin-" + /*subscription*/ ctx[11].list_id);
				attr_dev(label1, "class", "ml-2 block text-xs text-gray-700");
				add_location(label1, file$4, 153, 24, 6889);
				attr_dev(div9, "class", "flex items-center");
				add_location(div9, file$4, 145, 22, 6378);
				attr_dev(div10, "class", "space-y-2");
				add_location(div10, file$4, 132, 20, 5622);
				add_location(div11, file$4, 130, 18, 5495);
				attr_dev(h51, "class", "block text-xs font-medium text-gray-700 mb-2");
				add_location(h51, file$4, 162, 20, 7249);
				attr_dev(input2, "id", input2_id_value = "auto-update-" + /*subscription*/ ctx[11].list_id);
				attr_dev(input2, "type", "checkbox");
				input2.checked = input2_checked_value = /*subscription*/ ctx[11].auto_update;
				attr_dev(input2, "class", "focus:ring-indigo-500 h-3 w-3 text-indigo-600 border-gray-300 rounded");
				add_location(input2, file$4, 165, 24, 7465);
				attr_dev(div12, "class", "flex items-center h-4");
				add_location(div12, file$4, 164, 22, 7405);
				attr_dev(label2, "for", label2_for_value = "auto-update-" + /*subscription*/ ctx[11].list_id);
				attr_dev(label2, "class", "font-medium text-gray-700");
				add_location(label2, file$4, 174, 24, 7996);
				attr_dev(p3, "class", "text-gray-500");
				add_location(p3, file$4, 177, 24, 8187);
				attr_dev(div13, "class", "ml-2 text-xs");
				add_location(div13, file$4, 173, 22, 7945);
				attr_dev(div14, "class", "flex items-start");
				add_location(div14, file$4, 163, 20, 7352);
				add_location(div15, file$4, 161, 18, 7223);
				attr_dev(div16, "class", "grid grid-cols-1 gap-4 sm:grid-cols-2");
				add_location(div16, file$4, 128, 16, 5382);
				attr_dev(div17, "class", "mt-4 pt-4 border-t border-gray-200");
				add_location(div17, file$4, 127, 14, 5317);
				attr_dev(div18, "class", "px-4 py-4 sm:px-6");
				add_location(div18, file$4, 76, 12, 2679);
				add_location(li, file$4, 75, 10, 2662);
			},
			m: function mount(target, anchor) {
				insert_dev(target, li, anchor);
				append_dev(li, div18);
				append_dev(div18, div7);
				append_dev(div7, div5);
				append_dev(div5, div1);
				append_dev(div1, div0);
				append_dev(div0, svg);
				append_dev(svg, path);
				append_dev(div5, t0);
				append_dev(div5, div4);
				append_dev(div4, div2);
				append_dev(div2, p0);
				append_dev(p0, t1);
				append_dev(div2, t2);
				append_dev(div2, span);
				append_dev(span, t3);
				append_dev(div4, t4);
				append_dev(div4, div3);
				append_dev(div3, p1);
				append_dev(p1, t5);
				append_dev(p1, t6);
				append_dev(p1, t7);
				append_dev(p1, t8);
				append_dev(p1, t9);
				if (if_block) if_block.m(p1, null);
				append_dev(div4, t10);
				append_dev(div4, p2);
				append_dev(p2, t11);
				append_dev(div7, t12);
				append_dev(div7, div6);
				append_dev(div6, button0);
				append_dev(div6, t14);
				append_dev(div6, button1);
				append_dev(div18, t16);
				append_dev(div18, div17);
				append_dev(div17, div16);
				append_dev(div16, div11);
				append_dev(div11, h50);
				append_dev(div11, t18);
				append_dev(div11, div10);
				append_dev(div10, div8);
				append_dev(div8, input0);
				append_dev(div8, t19);
				append_dev(div8, label0);
				append_dev(label0, t20);
				append_dev(div10, t21);
				append_dev(div10, div9);
				append_dev(div9, input1);
				append_dev(div9, t22);
				append_dev(div9, label1);
				append_dev(label1, t23);
				append_dev(label1, t24);
				append_dev(div16, t25);
				append_dev(div16, div15);
				append_dev(div15, h51);
				append_dev(div15, t27);
				append_dev(div15, div14);
				append_dev(div14, div12);
				append_dev(div12, input2);
				append_dev(div14, t28);
				append_dev(div14, div13);
				append_dev(div13, label2);
				append_dev(label2, t29);
				append_dev(div13, t30);
				append_dev(div13, p3);
				append_dev(li, t32);

				if (!mounted) {
					dispose = [
						listen_dev(button0, "click", click_handler, false, false, false, false),
						listen_dev(button1, "click", click_handler_1, false, false, false, false),
						listen_dev(input0, "change", change_handler, false, false, false, false),
						listen_dev(input1, "change", change_handler_1, false, false, false, false),
						listen_dev(input2, "change", change_handler_2, false, false, false, false)
					];

					mounted = true;
				}
			},
			p: function update(new_ctx, dirty) {
				ctx = new_ctx;
				if (dirty & /*subscriptions*/ 1 && t1_value !== (t1_value = /*subscription*/ ctx[11].list.name + "")) set_data_dev(t1, t1_value);
				if (dirty & /*subscriptions*/ 1 && t3_value !== (t3_value = /*subscription*/ ctx[11].list.update_cadence + "")) set_data_dev(t3, t3_value);

				if (dirty & /*subscriptions*/ 1 && span_class_value !== (span_class_value = "ml-2 inline-flex items-center px-2 py-0.5 rounded text-xs font-medium " + getUpdateCadenceColor(/*subscription*/ ctx[11].list.update_cadence))) {
					attr_dev(span, "class", span_class_value);
				}

				if (dirty & /*subscriptions*/ 1 && t6_value !== (t6_value = formatDate$1(/*subscription*/ ctx[11].created_at) + "")) set_data_dev(t6, t6_value);
				if (dirty & /*subscriptions*/ 1 && t8_value !== (t8_value = /*subscription*/ ctx[11].list.version + "")) set_data_dev(t8, t8_value);

				if (/*subscription*/ ctx[11].version_pinned) {
					if (if_block) {
						if_block.p(ctx, dirty);
					} else {
						if_block = create_if_block_1$4(ctx);
						if_block.c();
						if_block.m(p1, null);
					}
				} else if (if_block) {
					if_block.d(1);
					if_block = null;
				}

				if (dirty & /*subscriptions*/ 1 && t11_value !== (t11_value = /*subscription*/ ctx[11].list.description + "")) set_data_dev(t11, t11_value);

				if (dirty & /*subscriptions*/ 1 && input0_id_value !== (input0_id_value = "auto-" + /*subscription*/ ctx[11].list_id)) {
					attr_dev(input0, "id", input0_id_value);
				}

				if (dirty & /*subscriptions*/ 1 && input0_checked_value !== (input0_checked_value = !/*subscription*/ ctx[11].version_pinned)) {
					prop_dev(input0, "checked", input0_checked_value);
				}

				if (dirty & /*subscriptions*/ 1 && label0_for_value !== (label0_for_value = "auto-" + /*subscription*/ ctx[11].list_id)) {
					attr_dev(label0, "for", label0_for_value);
				}

				if (dirty & /*subscriptions*/ 1 && input1_id_value !== (input1_id_value = "pin-" + /*subscription*/ ctx[11].list_id)) {
					attr_dev(input1, "id", input1_id_value);
				}

				if (dirty & /*subscriptions*/ 1 && input1_checked_value !== (input1_checked_value = !!/*subscription*/ ctx[11].version_pinned)) {
					prop_dev(input1, "checked", input1_checked_value);
				}

				if (dirty & /*subscriptions*/ 1 && t24_value !== (t24_value = /*subscription*/ ctx[11].list.version + "")) set_data_dev(t24, t24_value);

				if (dirty & /*subscriptions*/ 1 && label1_for_value !== (label1_for_value = "pin-" + /*subscription*/ ctx[11].list_id)) {
					attr_dev(label1, "for", label1_for_value);
				}

				if (dirty & /*subscriptions*/ 1 && input2_id_value !== (input2_id_value = "auto-update-" + /*subscription*/ ctx[11].list_id)) {
					attr_dev(input2, "id", input2_id_value);
				}

				if (dirty & /*subscriptions*/ 1 && input2_checked_value !== (input2_checked_value = /*subscription*/ ctx[11].auto_update)) {
					prop_dev(input2, "checked", input2_checked_value);
				}

				if (dirty & /*subscriptions*/ 1 && label2_for_value !== (label2_for_value = "auto-update-" + /*subscription*/ ctx[11].list_id)) {
					attr_dev(label2, "for", label2_for_value);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(li);
				}

				if (if_block) if_block.d();
				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_each_block$1.name,
			type: "each",
			source: "(67:8) {#each subscriptions as subscription}",
			ctx
		});

		return block;
	}

	function create_fragment$4(ctx) {
		let div6;
		let div0;
		let h30;
		let t1;
		let p;
		let t3;
		let t4;
		let div5;
		let div4;
		let div1;
		let svg;
		let path;
		let t5;
		let div3;
		let h31;
		let t7;
		let div2;
		let ul;
		let li0;
		let strong0;
		let t9;
		let t10;
		let li1;
		let strong1;
		let t12;
		let t13;
		let li2;
		let strong2;
		let t15;
		let t16;
		let li3;
		let strong3;
		let t18;

		function select_block_type(ctx, dirty) {
			if (/*subscriptions*/ ctx[0].length === 0) return create_if_block$4;
			return create_else_block$4;
		}

		let current_block_type = select_block_type(ctx);
		let if_block = current_block_type(ctx);

		const block = {
			c: function create() {
				div6 = element("div");
				div0 = element("div");
				h30 = element("h3");
				h30.textContent = "My Subscriptions";
				t1 = space();
				p = element("p");
				p.textContent = "Manage your community list subscriptions and update preferences.";
				t3 = space();
				if_block.c();
				t4 = space();
				div5 = element("div");
				div4 = element("div");
				div1 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t5 = space();
				div3 = element("div");
				h31 = element("h3");
				h31.textContent = "Managing Your Subscriptions";
				t7 = space();
				div2 = element("div");
				ul = element("ul");
				li0 = element("li");
				strong0 = element("strong");
				strong0.textContent = "Auto-update:";
				t9 = text(" Automatically apply changes when lists are updated");
				t10 = space();
				li1 = element("li");
				strong1 = element("strong");
				strong1.textContent = "Version pinning:";
				t12 = text(" Stay on a specific version to avoid unexpected changes");
				t13 = space();
				li2 = element("li");
				strong2 = element("strong");
				strong2.textContent = "Notifications:";
				t15 = text(" Get notified about list updates and changes");
				t16 = space();
				li3 = element("li");
				strong3 = element("strong");
				strong3.textContent = "Impact preview:";
				t18 = text(" See what changes will be made before they're applied");
				attr_dev(h30, "class", "text-lg font-medium text-gray-900");
				add_location(h30, file$4, 53, 4, 1586);
				attr_dev(p, "class", "text-sm text-gray-500");
				add_location(p, file$4, 54, 4, 1658);
				add_location(div0, file$4, 52, 2, 1576);
				attr_dev(path, "fill-rule", "evenodd");
				attr_dev(path, "d", "M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z");
				attr_dev(path, "clip-rule", "evenodd");
				add_location(path, file$4, 197, 10, 8771);
				attr_dev(svg, "class", "h-5 w-5 text-blue-400");
				attr_dev(svg, "viewBox", "0 0 20 20");
				attr_dev(svg, "fill", "currentColor");
				add_location(svg, file$4, 196, 8, 8685);
				attr_dev(div1, "class", "flex-shrink-0");
				add_location(div1, file$4, 195, 6, 8649);
				attr_dev(h31, "class", "text-sm font-medium text-blue-800");
				add_location(h31, file$4, 201, 8, 9014);
				add_location(strong0, file$4, 206, 16, 9233);
				add_location(li0, file$4, 206, 12, 9229);
				add_location(strong1, file$4, 207, 16, 9335);
				add_location(li1, file$4, 207, 12, 9331);
				add_location(strong2, file$4, 208, 16, 9445);
				add_location(li2, file$4, 208, 12, 9441);
				add_location(strong3, file$4, 209, 16, 9542);
				add_location(li3, file$4, 209, 12, 9538);
				attr_dev(ul, "class", "list-disc list-inside space-y-1");
				add_location(ul, file$4, 205, 10, 9172);
				attr_dev(div2, "class", "mt-2 text-sm text-blue-700");
				add_location(div2, file$4, 204, 8, 9121);
				attr_dev(div3, "class", "ml-3");
				add_location(div3, file$4, 200, 6, 8987);
				attr_dev(div4, "class", "flex");
				add_location(div4, file$4, 194, 4, 8624);
				attr_dev(div5, "class", "bg-blue-50 border border-blue-200 rounded-md p-4");
				add_location(div5, file$4, 193, 2, 8557);
				attr_dev(div6, "class", "space-y-6");
				add_location(div6, file$4, 51, 0, 1550);
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				insert_dev(target, div6, anchor);
				append_dev(div6, div0);
				append_dev(div0, h30);
				append_dev(div0, t1);
				append_dev(div0, p);
				append_dev(div6, t3);
				if_block.m(div6, null);
				append_dev(div6, t4);
				append_dev(div6, div5);
				append_dev(div5, div4);
				append_dev(div4, div1);
				append_dev(div1, svg);
				append_dev(svg, path);
				append_dev(div4, t5);
				append_dev(div4, div3);
				append_dev(div3, h31);
				append_dev(div3, t7);
				append_dev(div3, div2);
				append_dev(div2, ul);
				append_dev(ul, li0);
				append_dev(li0, strong0);
				append_dev(li0, t9);
				append_dev(ul, t10);
				append_dev(ul, li1);
				append_dev(li1, strong1);
				append_dev(li1, t12);
				append_dev(ul, t13);
				append_dev(ul, li2);
				append_dev(li2, strong2);
				append_dev(li2, t15);
				append_dev(ul, t16);
				append_dev(ul, li3);
				append_dev(li3, strong3);
				append_dev(li3, t18);
			},
			p: function update(ctx, [dirty]) {
				if (current_block_type === (current_block_type = select_block_type(ctx)) && if_block) {
					if_block.p(ctx, dirty);
				} else {
					if_block.d(1);
					if_block = current_block_type(ctx);

					if (if_block) {
						if_block.c();
						if_block.m(div6, t4);
					}
				}
			},
			i: noop,
			o: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div6);
				}

				if_block.d();
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

	function formatDate$1(dateString) {
		return new Date(dateString).toLocaleDateString();
	}

	function getUpdateCadenceColor(cadence) {
		switch (cadence.toLowerCase()) {
			case 'daily':
				return 'text-red-600 bg-red-100';
			case 'weekly':
				return 'text-yellow-600 bg-yellow-100';
			case 'monthly':
				return 'text-green-600 bg-green-100';
			case 'as-needed':
				return 'text-blue-600 bg-blue-100';
			default:
				return 'text-gray-600 bg-gray-100';
		}
	}

	function instance$4($$self, $$props, $$invalidate) {
		let subscriptions;
		let $communityStore;
		validate_store(communityStore, 'communityStore');
		component_subscribe($$self, communityStore, $$value => $$invalidate(5, $communityStore = $$value));
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('MySubscriptions', slots, []);

		async function updateSubscription(listId, versionPinned, autoUpdate) {
			const result = await communityActions.updateSubscription(listId, versionPinned, autoUpdate);

			if (!result.success) {
				alert(`Failed to update subscription: ${result.message}`);
			}
		}

		async function unsubscribe(listId, listName) {
			const confirmed = confirm(`Are you sure you want to unsubscribe from "${listName}"?`);

			if (confirmed) {
				const result = await communityActions.unsubscribe(listId);

				if (!result.success) {
					alert(`Failed to unsubscribe: ${result.message}`);
				}
			}
		}

		async function viewListDetails(listId) {
			await communityActions.fetchListDetails(listId);
		}

		function handleAutoUpdateChange(listId, versionPinned, event) {
			const target = event.target;
			updateSubscription(listId, versionPinned, target.checked);
		}

		const writable_props = [];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<MySubscriptions> was created with unknown prop '${key}'`);
		});

		const click_handler = subscription => viewListDetails(subscription.list_id);
		const click_handler_1 = subscription => unsubscribe(subscription.list_id, subscription.list.name);
		const change_handler = subscription => updateSubscription(subscription.list_id, undefined, subscription.auto_update);
		const change_handler_1 = subscription => updateSubscription(subscription.list_id, subscription.list.version, subscription.auto_update);
		const change_handler_2 = (subscription, e) => handleAutoUpdateChange(subscription.list_id, subscription.version_pinned, e);

		$$self.$capture_state = () => ({
			communityActions,
			communityStore,
			updateSubscription,
			unsubscribe,
			viewListDetails,
			formatDate: formatDate$1,
			getUpdateCadenceColor,
			handleAutoUpdateChange,
			subscriptions,
			$communityStore
		});

		$$self.$inject_state = $$props => {
			if ('subscriptions' in $$props) $$invalidate(0, subscriptions = $$props.subscriptions);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		$$self.$$.update = () => {
			if ($$self.$$.dirty & /*$communityStore*/ 32) {
				$$invalidate(0, subscriptions = $communityStore.subscriptions);
			}
		};

		return [
			subscriptions,
			updateSubscription,
			unsubscribe,
			viewListDetails,
			handleAutoUpdateChange,
			$communityStore,
			click_handler,
			click_handler_1,
			change_handler,
			change_handler_1,
			change_handler_2
		];
	}

	class MySubscriptions extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance$4, create_fragment$4, safe_not_equal, {});

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "MySubscriptions",
				options,
				id: create_fragment$4.name
			});
		}
	}

	/* src/lib/components/CommunityLists.svelte generated by Svelte v4.2.20 */
	const file$3 = "src/lib/components/CommunityLists.svelte";

	function get_each_context(ctx, list, i) {
		const child_ctx = ctx.slice();
		child_ctx[14] = list[i];
		return child_ctx;
	}

	// (51:2) {#if showCreateForm}
	function create_if_block_7$2(ctx) {
		let div;
		let h3;
		let t1;
		let createcommunitylist;
		let current;
		createcommunitylist = new CreateCommunityList({ $$inline: true });
		createcommunitylist.$on("listCreated", /*listCreated_handler*/ ctx[9]);

		const block = {
			c: function create() {
				div = element("div");
				h3 = element("h3");
				h3.textContent = "Create Community List";
				t1 = space();
				create_component(createcommunitylist.$$.fragment);
				attr_dev(h3, "class", "text-lg font-medium text-gray-900 mb-4");
				add_location(h3, file$3, 58, 6, 2031);
				attr_dev(div, "class", "mb-6 bg-white shadow rounded-lg p-6");
				add_location(div, file$3, 57, 4, 1975);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, h3);
				append_dev(div, t1);
				mount_component(createcommunitylist, div, null);
				current = true;
			},
			p: noop,
			i: function intro(local) {
				if (current) return;
				transition_in(createcommunitylist.$$.fragment, local);
				current = true;
			},
			o: function outro(local) {
				transition_out(createcommunitylist.$$.fragment, local);
				current = false;
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}

				destroy_component(createcommunitylist);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_7$2.name,
			type: "if",
			source: "(51:2) {#if showCreateForm}",
			ctx
		});

		return block;
	}

	// (182:42) 
	function create_if_block_6$2(ctx) {
		let mysubscriptions;
		let current;
		mysubscriptions = new MySubscriptions({ $$inline: true });

		const block = {
			c: function create() {
				create_component(mysubscriptions.$$.fragment);
			},
			m: function mount(target, anchor) {
				mount_component(mysubscriptions, target, anchor);
				current = true;
			},
			p: noop,
			i: function intro(local) {
				if (current) return;
				transition_in(mysubscriptions.$$.fragment, local);
				current = true;
			},
			o: function outro(local) {
				transition_out(mysubscriptions.$$.fragment, local);
				current = false;
			},
			d: function destroy(detaching) {
				destroy_component(mysubscriptions, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_6$2.name,
			type: "if",
			source: "(182:42) ",
			ctx
		});

		return block;
	}

	// (77:2) {#if activeTab === 'browse'}
	function create_if_block$3(ctx) {
		let current_block_type_index;
		let if_block;
		let if_block_anchor;
		let current;
		const if_block_creators = [create_if_block_1$3, create_else_block$3];
		const if_blocks = [];

		function select_block_type_1(ctx, dirty) {
			if (/*$communityStore*/ ctx[3].currentList) return 0;
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
			id: create_if_block$3.name,
			type: "if",
			source: "(77:2) {#if activeTab === 'browse'}",
			ctx
		});

		return block;
	}

	// (81:4) {:else}
	function create_else_block$3(ctx) {
		let div6;
		let div5;
		let div4;
		let div2;
		let label0;
		let t1;
		let div1;
		let div0;
		let svg;
		let path;
		let t2;
		let input;
		let input_value_value;
		let t3;
		let div3;
		let label1;
		let t5;
		let select;
		let option0;
		let option1;
		let option2;
		let option3;
		let option4;
		let option5;
		let t12;
		let current_block_type_index;
		let if_block;
		let current;
		let mounted;
		let dispose;
		const if_block_creators = [create_if_block_2$2, create_if_block_3$2, create_if_block_4$2, create_else_block_2$1];
		const if_blocks = [];

		function select_block_type_2(ctx, dirty) {
			if (/*$communityStore*/ ctx[3].isLoading) return 0;
			if (/*$communityStore*/ ctx[3].error) return 1;
			if (/*$filteredLists*/ ctx[4].length === 0) return 2;
			return 3;
		}

		current_block_type_index = select_block_type_2(ctx);
		if_block = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);

		const block = {
			c: function create() {
				div6 = element("div");
				div5 = element("div");
				div4 = element("div");
				div2 = element("div");
				label0 = element("label");
				label0.textContent = "Search lists";
				t1 = space();
				div1 = element("div");
				div0 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t2 = space();
				input = element("input");
				t3 = space();
				div3 = element("div");
				label1 = element("label");
				label1.textContent = "Sort by";
				t5 = space();
				select = element("select");
				option0 = element("option");
				option0.textContent = "Recently Updated";
				option1 = element("option");
				option1.textContent = "Newest First";
				option2 = element("option");
				option2.textContent = "Name A-Z";
				option3 = element("option");
				option3.textContent = "Name Z-A";
				option4 = element("option");
				option4.textContent = "Most Artists";
				option5 = element("option");
				option5.textContent = "Most Subscribers";
				t12 = space();
				if_block.c();
				attr_dev(label0, "for", "search");
				attr_dev(label0, "class", "sr-only");
				add_location(label0, file$3, 93, 14, 3430);
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z");
				add_location(path, file$3, 97, 20, 3751);
				attr_dev(svg, "class", "h-5 w-5 text-gray-400");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$3, 96, 18, 3641);
				attr_dev(div0, "class", "absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none");
				add_location(div0, file$3, 95, 16, 3540);
				attr_dev(input, "id", "search");
				attr_dev(input, "type", "text");
				attr_dev(input, "placeholder", "Search lists by name, description, or criteria...");
				input.value = input_value_value = /*$communityStore*/ ctx[3].searchQuery;
				attr_dev(input, "class", "block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md leading-5 bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm");
				add_location(input, file$3, 100, 16, 3936);
				attr_dev(div1, "class", "relative");
				add_location(div1, file$3, 94, 14, 3501);
				attr_dev(div2, "class", "flex-1");
				add_location(div2, file$3, 92, 12, 3395);
				attr_dev(label1, "for", "sort");
				attr_dev(label1, "class", "sr-only");
				add_location(label1, file$3, 112, 14, 4546);
				option0.__value = "updated_at:desc";
				set_input_value(option0, option0.__value);
				add_location(option0, file$3, 118, 16, 4888);
				option1.__value = "created_at:desc";
				set_input_value(option1, option1.__value);
				add_location(option1, file$3, 119, 16, 4962);
				option2.__value = "name:asc";
				set_input_value(option2, option2.__value);
				add_location(option2, file$3, 120, 16, 5032);
				option3.__value = "name:desc";
				set_input_value(option3, option3.__value);
				add_location(option3, file$3, 121, 16, 5091);
				option4.__value = "artist_count:desc";
				set_input_value(option4, option4.__value);
				add_location(option4, file$3, 122, 16, 5151);
				option5.__value = "subscriber_count:desc";
				set_input_value(option5, option5.__value);
				add_location(option5, file$3, 123, 16, 5223);
				attr_dev(select, "id", "sort");
				attr_dev(select, "class", "block w-full pl-3 pr-10 py-2 text-base border border-gray-300 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm rounded-md");
				add_location(select, file$3, 113, 14, 4610);
				attr_dev(div3, "class", "sm:w-48");
				add_location(div3, file$3, 111, 12, 4510);
				attr_dev(div4, "class", "flex flex-col sm:flex-row gap-4");
				add_location(div4, file$3, 91, 10, 3337);
				attr_dev(div5, "class", "bg-white shadow rounded-lg p-4");
				add_location(div5, file$3, 90, 8, 3282);
				attr_dev(div6, "class", "space-y-6");
				add_location(div6, file$3, 88, 6, 3215);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div6, anchor);
				append_dev(div6, div5);
				append_dev(div5, div4);
				append_dev(div4, div2);
				append_dev(div2, label0);
				append_dev(div2, t1);
				append_dev(div2, div1);
				append_dev(div1, div0);
				append_dev(div0, svg);
				append_dev(svg, path);
				append_dev(div1, t2);
				append_dev(div1, input);
				append_dev(div4, t3);
				append_dev(div4, div3);
				append_dev(div3, label1);
				append_dev(div3, t5);
				append_dev(div3, select);
				append_dev(select, option0);
				append_dev(select, option1);
				append_dev(select, option2);
				append_dev(select, option3);
				append_dev(select, option4);
				append_dev(select, option5);
				append_dev(div6, t12);
				if_blocks[current_block_type_index].m(div6, null);
				current = true;

				if (!mounted) {
					dispose = [
						listen_dev(input, "input", /*handleSearch*/ ctx[6], false, false, false, false),
						listen_dev(select, "change", /*handleSort*/ ctx[7], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (!current || dirty & /*$communityStore*/ 8 && input_value_value !== (input_value_value = /*$communityStore*/ ctx[3].searchQuery) && input.value !== input_value_value) {
					prop_dev(input, "value", input_value_value);
				}

				let previous_block_index = current_block_type_index;
				current_block_type_index = select_block_type_2(ctx);

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
					if_block.m(div6, null);
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
					detach_dev(div6);
				}

				if_blocks[current_block_type_index].d();
				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block$3.name,
			type: "else",
			source: "(81:4) {:else}",
			ctx
		});

		return block;
	}

	// (79:4) {#if $communityStore.currentList}
	function create_if_block_1$3(ctx) {
		let communitylistdetail;
		let current;
		communitylistdetail = new CommunityListDetail({ $$inline: true });

		const block = {
			c: function create() {
				create_component(communitylistdetail.$$.fragment);
			},
			m: function mount(target, anchor) {
				mount_component(communitylistdetail, target, anchor);
				current = true;
			},
			p: noop,
			i: function intro(local) {
				if (current) return;
				transition_in(communitylistdetail.$$.fragment, local);
				current = true;
			},
			o: function outro(local) {
				transition_out(communitylistdetail.$$.fragment, local);
				current = false;
			},
			d: function destroy(detaching) {
				destroy_component(communitylistdetail, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_1$3.name,
			type: "if",
			source: "(79:4) {#if $communityStore.currentList}",
			ctx
		});

		return block;
	}

	// (173:8) {:else}
	function create_else_block_2$1(ctx) {
		let div;
		let each_blocks = [];
		let each_1_lookup = new Map();
		let current;
		let each_value = ensure_array_like_dev(/*$filteredLists*/ ctx[4]);
		const get_key = ctx => /*list*/ ctx[14].id;
		validate_each_keys(ctx, each_value, get_each_context, get_key);

		for (let i = 0; i < each_value.length; i += 1) {
			let child_ctx = get_each_context(ctx, each_value, i);
			let key = get_key(child_ctx);
			each_1_lookup.set(key, each_blocks[i] = create_each_block(key, child_ctx));
		}

		const block = {
			c: function create() {
				div = element("div");

				for (let i = 0; i < each_blocks.length; i += 1) {
					each_blocks[i].c();
				}

				attr_dev(div, "class", "grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3");
				add_location(div, file$3, 179, 10, 8629);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);

				for (let i = 0; i < each_blocks.length; i += 1) {
					if (each_blocks[i]) {
						each_blocks[i].m(div, null);
					}
				}

				current = true;
			},
			p: function update(ctx, dirty) {
				if (dirty & /*$filteredLists*/ 16) {
					each_value = ensure_array_like_dev(/*$filteredLists*/ ctx[4]);
					group_outros();
					validate_each_keys(ctx, each_value, get_each_context, get_key);
					each_blocks = update_keyed_each(each_blocks, dirty, get_key, 1, ctx, each_value, each_1_lookup, div, outro_and_destroy_block, create_each_block, null, get_each_context);
					check_outros();
				}
			},
			i: function intro(local) {
				if (current) return;

				for (let i = 0; i < each_value.length; i += 1) {
					transition_in(each_blocks[i]);
				}

				current = true;
			},
			o: function outro(local) {
				for (let i = 0; i < each_blocks.length; i += 1) {
					transition_out(each_blocks[i]);
				}

				current = false;
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}

				for (let i = 0; i < each_blocks.length; i += 1) {
					each_blocks[i].d();
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block_2$1.name,
			type: "else",
			source: "(173:8) {:else}",
			ctx
		});

		return block;
	}

	// (146:46) 
	function create_if_block_4$2(ctx) {
		let div;

		function select_block_type_3(ctx, dirty) {
			if (/*$communityStore*/ ctx[3].lists.length === 0) return create_if_block_5$2;
			return create_else_block_1$2;
		}

		let current_block_type = select_block_type_3(ctx);
		let if_block = current_block_type(ctx);

		const block = {
			c: function create() {
				div = element("div");
				if_block.c();
				attr_dev(div, "class", "text-center py-12");
				add_location(div, file$3, 152, 10, 6704);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				if_block.m(div, null);
			},
			p: function update(ctx, dirty) {
				if (current_block_type === (current_block_type = select_block_type_3(ctx)) && if_block) {
					if_block.p(ctx, dirty);
				} else {
					if_block.d(1);
					if_block = current_block_type(ctx);

					if (if_block) {
						if_block.c();
						if_block.m(div, null);
					}
				}
			},
			i: noop,
			o: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}

				if_block.d();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_4$2.name,
			type: "if",
			source: "(146:46) ",
			ctx
		});

		return block;
	}

	// (133:40) 
	function create_if_block_3$2(ctx) {
		let div;
		let svg;
		let path;
		let t0;
		let p;
		let t1_value = /*$communityStore*/ ctx[3].error + "";
		let t1;
		let t2;
		let button;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				div = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				p = element("p");
				t1 = text(t1_value);
				t2 = space();
				button = element("button");
				button.textContent = "Try again";
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z");
				add_location(path, file$3, 141, 14, 6193);
				attr_dev(svg, "class", "mx-auto h-8 w-8 text-red-400");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$3, 140, 12, 6082);
				attr_dev(p, "class", "mt-2 text-sm text-red-600");
				add_location(p, file$3, 143, 12, 6351);
				attr_dev(button, "class", "mt-2 text-sm text-indigo-600 hover:text-indigo-500");
				add_location(button, file$3, 144, 12, 6428);
				attr_dev(div, "class", "text-center py-12");
				add_location(div, file$3, 139, 10, 6038);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, svg);
				append_dev(svg, path);
				append_dev(div, t0);
				append_dev(div, p);
				append_dev(p, t1);
				append_dev(div, t2);
				append_dev(div, button);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*click_handler_3*/ ctx[12], false, false, false, false);
					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (dirty & /*$communityStore*/ 8 && t1_value !== (t1_value = /*$communityStore*/ ctx[3].error + "")) set_data_dev(t1, t1_value);
			},
			i: noop,
			o: noop,
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
			id: create_if_block_3$2.name,
			type: "if",
			source: "(133:40) ",
			ctx
		});

		return block;
	}

	// (125:8) {#if $communityStore.isLoading}
	function create_if_block_2$2(ctx) {
		let div;
		let svg;
		let circle;
		let path;
		let t0;
		let p;

		const block = {
			c: function create() {
				div = element("div");
				svg = svg_element("svg");
				circle = svg_element("circle");
				path = svg_element("path");
				t0 = space();
				p = element("p");
				p.textContent = "Loading community lists...";
				attr_dev(circle, "class", "opacity-25");
				attr_dev(circle, "cx", "12");
				attr_dev(circle, "cy", "12");
				attr_dev(circle, "r", "10");
				attr_dev(circle, "stroke", "currentColor");
				attr_dev(circle, "stroke-width", "4");
				add_location(circle, file$3, 133, 14, 5588);
				attr_dev(path, "class", "opacity-75");
				attr_dev(path, "fill", "currentColor");
				attr_dev(path, "d", "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z");
				add_location(path, file$3, 134, 14, 5701);
				attr_dev(svg, "class", "animate-spin mx-auto h-8 w-8 text-gray-400");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				add_location(svg, file$3, 132, 12, 5485);
				attr_dev(p, "class", "mt-2 text-sm text-gray-500");
				add_location(p, file$3, 136, 12, 5901);
				attr_dev(div, "class", "text-center py-12");
				add_location(div, file$3, 131, 10, 5441);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, svg);
				append_dev(svg, circle);
				append_dev(svg, path);
				append_dev(div, t0);
				append_dev(div, p);
			},
			p: noop,
			i: noop,
			o: noop,
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
			source: "(125:8) {#if $communityStore.isLoading}",
			ctx
		});

		return block;
	}

	// (175:12) {#each $filteredLists as list (list.id)}
	function create_each_block(key_1, ctx) {
		let first;
		let communitylistcard;
		let current;

		communitylistcard = new CommunityListCard({
				props: { list: /*list*/ ctx[14] },
				$$inline: true
			});

		const block = {
			key: key_1,
			first: null,
			c: function create() {
				first = empty();
				create_component(communitylistcard.$$.fragment);
				this.first = first;
			},
			m: function mount(target, anchor) {
				insert_dev(target, first, anchor);
				mount_component(communitylistcard, target, anchor);
				current = true;
			},
			p: function update(new_ctx, dirty) {
				ctx = new_ctx;
				const communitylistcard_changes = {};
				if (dirty & /*$filteredLists*/ 16) communitylistcard_changes.list = /*list*/ ctx[14];
				communitylistcard.$set(communitylistcard_changes);
			},
			i: function intro(local) {
				if (current) return;
				transition_in(communitylistcard.$$.fragment, local);
				current = true;
			},
			o: function outro(local) {
				transition_out(communitylistcard.$$.fragment, local);
				current = false;
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(first);
				}

				destroy_component(communitylistcard, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_each_block.name,
			type: "each",
			source: "(175:12) {#each $filteredLists as list (list.id)}",
			ctx
		});

		return block;
	}

	// (165:12) {:else}
	function create_else_block_1$2(ctx) {
		let svg;
		let path;
		let t0;
		let h3;
		let t2;
		let p;

		const block = {
			c: function create() {
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				h3 = element("h3");
				h3.textContent = "No lists match your search";
				t2 = space();
				p = element("p");
				p.textContent = "Try adjusting your search terms or filters.";
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z");
				add_location(path, file$3, 172, 16, 8229);
				attr_dev(svg, "class", "mx-auto h-12 w-12 text-gray-400");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$3, 171, 14, 8113);
				attr_dev(h3, "class", "mt-2 text-sm font-medium text-gray-900");
				add_location(h3, file$3, 174, 14, 8385);
				attr_dev(p, "class", "mt-1 text-sm text-gray-500");
				add_location(p, file$3, 175, 14, 8482);
			},
			m: function mount(target, anchor) {
				insert_dev(target, svg, anchor);
				append_dev(svg, path);
				insert_dev(target, t0, anchor);
				insert_dev(target, h3, anchor);
				insert_dev(target, t2, anchor);
				insert_dev(target, p, anchor);
			},
			p: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(svg);
					detach_dev(t0);
					detach_dev(h3);
					detach_dev(t2);
					detach_dev(p);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block_1$2.name,
			type: "else",
			source: "(165:12) {:else}",
			ctx
		});

		return block;
	}

	// (148:12) {#if $communityStore.lists.length === 0}
	function create_if_block_5$2(ctx) {
		let svg0;
		let path0;
		let t0;
		let h3;
		let t2;
		let p;
		let t4;
		let div;
		let button;
		let svg1;
		let path1;
		let t5;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				svg0 = svg_element("svg");
				path0 = svg_element("path");
				t0 = space();
				h3 = element("h3");
				h3.textContent = "No community lists yet";
				t2 = space();
				p = element("p");
				p.textContent = "Be the first to create a community list.";
				t4 = space();
				div = element("div");
				button = element("button");
				svg1 = svg_element("svg");
				path1 = svg_element("path");
				t5 = text("\n                  Create your first list");
				attr_dev(path0, "stroke-linecap", "round");
				attr_dev(path0, "stroke-linejoin", "round");
				attr_dev(path0, "stroke-width", "2");
				attr_dev(path0, "d", "M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10");
				add_location(path0, file$3, 155, 16, 6919);
				attr_dev(svg0, "class", "mx-auto h-12 w-12 text-gray-400");
				attr_dev(svg0, "fill", "none");
				attr_dev(svg0, "viewBox", "0 0 24 24");
				attr_dev(svg0, "stroke", "currentColor");
				add_location(svg0, file$3, 154, 14, 6803);
				attr_dev(h3, "class", "mt-2 text-sm font-medium text-gray-900");
				add_location(h3, file$3, 157, 14, 7182);
				attr_dev(p, "class", "mt-1 text-sm text-gray-500");
				add_location(p, file$3, 158, 14, 7275);
				attr_dev(path1, "stroke-linecap", "round");
				attr_dev(path1, "stroke-linejoin", "round");
				attr_dev(path1, "stroke-width", "2");
				attr_dev(path1, "d", "M12 6v6m0 0v6m0-6h6m-6 0H6");
				add_location(path1, file$3, 165, 20, 7862);
				attr_dev(svg1, "class", "-ml-1 mr-2 h-5 w-5");
				attr_dev(svg1, "fill", "none");
				attr_dev(svg1, "viewBox", "0 0 24 24");
				attr_dev(svg1, "stroke", "currentColor");
				add_location(svg1, file$3, 164, 18, 7755);
				attr_dev(button, "class", "inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500");
				add_location(button, file$3, 160, 16, 7407);
				attr_dev(div, "class", "mt-6");
				add_location(div, file$3, 159, 14, 7372);
			},
			m: function mount(target, anchor) {
				insert_dev(target, svg0, anchor);
				append_dev(svg0, path0);
				insert_dev(target, t0, anchor);
				insert_dev(target, h3, anchor);
				insert_dev(target, t2, anchor);
				insert_dev(target, p, anchor);
				insert_dev(target, t4, anchor);
				insert_dev(target, div, anchor);
				append_dev(div, button);
				append_dev(button, svg1);
				append_dev(svg1, path1);
				append_dev(button, t5);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*click_handler_4*/ ctx[13], false, false, false, false);
					mounted = true;
				}
			},
			p: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(svg0);
					detach_dev(t0);
					detach_dev(h3);
					detach_dev(t2);
					detach_dev(p);
					detach_dev(t4);
					detach_dev(div);
				}

				mounted = false;
				dispose();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_5$2.name,
			type: "if",
			source: "(148:12) {#if $communityStore.lists.length === 0}",
			ctx
		});

		return block;
	}

	function create_fragment$3(ctx) {
		let div9;
		let div2;
		let div1;
		let div0;
		let h2;
		let t1;
		let p0;
		let t3;
		let button0;
		let svg0;
		let path0;
		let t4;
		let t5;
		let t6;
		let div3;
		let nav;
		let button1;
		let t7;
		let button1_class_value;
		let t8;
		let button2;
		let t9;
		let t10_value = /*$subscribedListIds*/ ctx[2].size + "";
		let t10;
		let t11;
		let button2_class_value;
		let t12;
		let current_block_type_index;
		let if_block1;
		let t13;
		let div8;
		let div7;
		let div4;
		let svg1;
		let path1;
		let t14;
		let div6;
		let h3;
		let t16;
		let div5;
		let p1;
		let t18;
		let ul;
		let li0;
		let t20;
		let li1;
		let t22;
		let li2;
		let t24;
		let li3;
		let current;
		let mounted;
		let dispose;
		let if_block0 = /*showCreateForm*/ ctx[1] && create_if_block_7$2(ctx);
		const if_block_creators = [create_if_block$3, create_if_block_6$2];
		const if_blocks = [];

		function select_block_type(ctx, dirty) {
			if (/*activeTab*/ ctx[0] === 'browse') return 0;
			if (/*activeTab*/ ctx[0] === 'subscriptions') return 1;
			return -1;
		}

		if (~(current_block_type_index = select_block_type(ctx))) {
			if_block1 = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);
		}

		const block = {
			c: function create() {
				div9 = element("div");
				div2 = element("div");
				div1 = element("div");
				div0 = element("div");
				h2 = element("h2");
				h2.textContent = "Community Lists";
				t1 = space();
				p0 = element("p");
				p0.textContent = "Discover and subscribe to community-curated blocklists.";
				t3 = space();
				button0 = element("button");
				svg0 = svg_element("svg");
				path0 = svg_element("path");
				t4 = text("\n        Create List");
				t5 = space();
				if (if_block0) if_block0.c();
				t6 = space();
				div3 = element("div");
				nav = element("nav");
				button1 = element("button");
				t7 = text("Browse Lists");
				t8 = space();
				button2 = element("button");
				t9 = text("My Subscriptions (");
				t10 = text(t10_value);
				t11 = text(")");
				t12 = space();
				if (if_block1) if_block1.c();
				t13 = space();
				div8 = element("div");
				div7 = element("div");
				div4 = element("div");
				svg1 = svg_element("svg");
				path1 = svg_element("path");
				t14 = space();
				div6 = element("div");
				h3 = element("h3");
				h3.textContent = "About Community Lists";
				t16 = space();
				div5 = element("div");
				p1 = element("p");
				p1.textContent = "Community lists are curated blocklists created and maintained by other users. \n            Each list has clear criteria and governance processes to ensure quality and transparency.";
				t18 = space();
				ul = element("ul");
				li0 = element("li");
				li0.textContent = "Subscribe to lists that match your preferences";
				t20 = space();
				li1 = element("li");
				li1.textContent = "Pin specific versions or enable auto-updates";
				t22 = space();
				li2 = element("li");
				li2.textContent = "Preview impact before subscribing";
				t24 = space();
				li3 = element("li");
				li3.textContent = "Appeal decisions through structured processes";
				attr_dev(h2, "class", "text-2xl font-bold text-gray-900");
				add_location(h2, file$3, 38, 8, 1117);
				attr_dev(p0, "class", "mt-1 text-sm text-gray-600");
				add_location(p0, file$3, 39, 8, 1191);
				add_location(div0, file$3, 37, 6, 1103);
				attr_dev(path0, "stroke-linecap", "round");
				attr_dev(path0, "stroke-linejoin", "round");
				attr_dev(path0, "stroke-width", "2");
				attr_dev(path0, "d", "M12 6v6m0 0v6m0-6h6m-6 0H6");
				add_location(path0, file$3, 48, 10, 1744);
				attr_dev(svg0, "class", "-ml-1 mr-2 h-5 w-5");
				attr_dev(svg0, "fill", "none");
				attr_dev(svg0, "viewBox", "0 0 24 24");
				attr_dev(svg0, "stroke", "currentColor");
				add_location(svg0, file$3, 47, 8, 1647);
				attr_dev(button0, "class", "inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500");
				add_location(button0, file$3, 43, 6, 1328);
				attr_dev(div1, "class", "flex justify-between items-center");
				add_location(div1, file$3, 36, 4, 1049);
				attr_dev(div2, "class", "mb-6");
				add_location(div2, file$3, 35, 2, 1026);

				attr_dev(button1, "class", button1_class_value = "py-4 px-1 border-b-2 font-medium text-sm " + (/*activeTab*/ ctx[0] === 'browse'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'));

				add_location(button1, file$3, 66, 6, 2344);

				attr_dev(button2, "class", button2_class_value = "py-4 px-1 border-b-2 font-medium text-sm " + (/*activeTab*/ ctx[0] === 'subscriptions'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'));

				add_location(button2, file$3, 72, 6, 2650);
				attr_dev(nav, "class", "flex space-x-8 px-6");
				attr_dev(nav, "aria-label", "Tabs");
				add_location(nav, file$3, 65, 4, 2286);
				attr_dev(div3, "class", "bg-white shadow-sm rounded-lg mb-6");
				add_location(div3, file$3, 64, 2, 2233);
				attr_dev(path1, "fill-rule", "evenodd");
				attr_dev(path1, "d", "M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z");
				attr_dev(path1, "clip-rule", "evenodd");
				add_location(path1, file$3, 196, 10, 9183);
				attr_dev(svg1, "class", "h-5 w-5 text-blue-400");
				attr_dev(svg1, "viewBox", "0 0 20 20");
				attr_dev(svg1, "fill", "currentColor");
				add_location(svg1, file$3, 195, 8, 9097);
				attr_dev(div4, "class", "flex-shrink-0");
				add_location(div4, file$3, 194, 6, 9061);
				attr_dev(h3, "class", "text-sm font-medium text-blue-800");
				add_location(h3, file$3, 200, 8, 9426);
				add_location(p1, file$3, 204, 10, 9578);
				add_location(li0, file$3, 209, 12, 9862);
				add_location(li1, file$3, 210, 12, 9930);
				add_location(li2, file$3, 211, 12, 9996);
				add_location(li3, file$3, 212, 12, 10051);
				attr_dev(ul, "class", "list-disc list-inside mt-2 space-y-1");
				add_location(ul, file$3, 208, 10, 9800);
				attr_dev(div5, "class", "mt-2 text-sm text-blue-700");
				add_location(div5, file$3, 203, 8, 9527);
				attr_dev(div6, "class", "ml-3");
				add_location(div6, file$3, 199, 6, 9399);
				attr_dev(div7, "class", "flex");
				add_location(div7, file$3, 193, 4, 9036);
				attr_dev(div8, "class", "mt-8 bg-blue-50 border border-blue-200 rounded-md p-4");
				add_location(div8, file$3, 192, 2, 8964);
				attr_dev(div9, "class", "px-4 py-6 sm:px-0");
				add_location(div9, file$3, 34, 0, 992);
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				insert_dev(target, div9, anchor);
				append_dev(div9, div2);
				append_dev(div2, div1);
				append_dev(div1, div0);
				append_dev(div0, h2);
				append_dev(div0, t1);
				append_dev(div0, p0);
				append_dev(div1, t3);
				append_dev(div1, button0);
				append_dev(button0, svg0);
				append_dev(svg0, path0);
				append_dev(button0, t4);
				append_dev(div9, t5);
				if (if_block0) if_block0.m(div9, null);
				append_dev(div9, t6);
				append_dev(div9, div3);
				append_dev(div3, nav);
				append_dev(nav, button1);
				append_dev(button1, t7);
				append_dev(nav, t8);
				append_dev(nav, button2);
				append_dev(button2, t9);
				append_dev(button2, t10);
				append_dev(button2, t11);
				append_dev(div9, t12);

				if (~current_block_type_index) {
					if_blocks[current_block_type_index].m(div9, null);
				}

				append_dev(div9, t13);
				append_dev(div9, div8);
				append_dev(div8, div7);
				append_dev(div7, div4);
				append_dev(div4, svg1);
				append_dev(svg1, path1);
				append_dev(div7, t14);
				append_dev(div7, div6);
				append_dev(div6, h3);
				append_dev(div6, t16);
				append_dev(div6, div5);
				append_dev(div5, p1);
				append_dev(div5, t18);
				append_dev(div5, ul);
				append_dev(ul, li0);
				append_dev(ul, t20);
				append_dev(ul, li1);
				append_dev(ul, t22);
				append_dev(ul, li2);
				append_dev(ul, t24);
				append_dev(ul, li3);
				current = true;

				if (!mounted) {
					dispose = [
						listen_dev(button0, "click", /*click_handler*/ ctx[8], false, false, false, false),
						listen_dev(button1, "click", /*click_handler_1*/ ctx[10], false, false, false, false),
						listen_dev(button2, "click", /*click_handler_2*/ ctx[11], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, [dirty]) {
				if (/*showCreateForm*/ ctx[1]) {
					if (if_block0) {
						if_block0.p(ctx, dirty);

						if (dirty & /*showCreateForm*/ 2) {
							transition_in(if_block0, 1);
						}
					} else {
						if_block0 = create_if_block_7$2(ctx);
						if_block0.c();
						transition_in(if_block0, 1);
						if_block0.m(div9, t6);
					}
				} else if (if_block0) {
					group_outros();

					transition_out(if_block0, 1, 1, () => {
						if_block0 = null;
					});

					check_outros();
				}

				if (!current || dirty & /*activeTab*/ 1 && button1_class_value !== (button1_class_value = "py-4 px-1 border-b-2 font-medium text-sm " + (/*activeTab*/ ctx[0] === 'browse'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'))) {
					attr_dev(button1, "class", button1_class_value);
				}

				if ((!current || dirty & /*$subscribedListIds*/ 4) && t10_value !== (t10_value = /*$subscribedListIds*/ ctx[2].size + "")) set_data_dev(t10, t10_value);

				if (!current || dirty & /*activeTab*/ 1 && button2_class_value !== (button2_class_value = "py-4 px-1 border-b-2 font-medium text-sm " + (/*activeTab*/ ctx[0] === 'subscriptions'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'))) {
					attr_dev(button2, "class", button2_class_value);
				}

				let previous_block_index = current_block_type_index;
				current_block_type_index = select_block_type(ctx);

				if (current_block_type_index === previous_block_index) {
					if (~current_block_type_index) {
						if_blocks[current_block_type_index].p(ctx, dirty);
					}
				} else {
					if (if_block1) {
						group_outros();

						transition_out(if_blocks[previous_block_index], 1, 1, () => {
							if_blocks[previous_block_index] = null;
						});

						check_outros();
					}

					if (~current_block_type_index) {
						if_block1 = if_blocks[current_block_type_index];

						if (!if_block1) {
							if_block1 = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);
							if_block1.c();
						} else {
							if_block1.p(ctx, dirty);
						}

						transition_in(if_block1, 1);
						if_block1.m(div9, t13);
					} else {
						if_block1 = null;
					}
				}
			},
			i: function intro(local) {
				if (current) return;
				transition_in(if_block0);
				transition_in(if_block1);
				current = true;
			},
			o: function outro(local) {
				transition_out(if_block0);
				transition_out(if_block1);
				current = false;
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div9);
				}

				if (if_block0) if_block0.d();

				if (~current_block_type_index) {
					if_blocks[current_block_type_index].d();
				}

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

	function instance$3($$self, $$props, $$invalidate) {
		let $subscribedListIds;
		let $communityStore;
		let $filteredLists;
		validate_store(subscribedListIds, 'subscribedListIds');
		component_subscribe($$self, subscribedListIds, $$value => $$invalidate(2, $subscribedListIds = $$value));
		validate_store(communityStore, 'communityStore');
		component_subscribe($$self, communityStore, $$value => $$invalidate(3, $communityStore = $$value));
		validate_store(filteredLists, 'filteredLists');
		component_subscribe($$self, filteredLists, $$value => $$invalidate(4, $filteredLists = $$value));
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('CommunityLists', slots, []);
		let activeTab = 'browse';
		let showCreateForm = false;

		onMount(async () => {
			await communityActions.fetchLists();
			await communityActions.fetchSubscriptions();
		});

		function setActiveTab(tab) {
			$$invalidate(0, activeTab = tab);
			$$invalidate(1, showCreateForm = false);
			communityActions.clearCurrentList();
		}

		function handleSearch(event) {
			const target = event.target;
			communityActions.updateSearch(target.value);
		}

		function handleSort(event) {
			const target = event.target;
			const [sortBy, sortOrder] = target.value.split(':');
			communityActions.updateSort(sortBy, sortOrder);
		}

		const writable_props = [];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<CommunityLists> was created with unknown prop '${key}'`);
		});

		const click_handler = () => $$invalidate(1, showCreateForm = !showCreateForm);
		const listCreated_handler = () => $$invalidate(1, showCreateForm = false);
		const click_handler_1 = () => setActiveTab('browse');
		const click_handler_2 = () => setActiveTab('subscriptions');
		const click_handler_3 = () => communityActions.fetchLists();
		const click_handler_4 = () => $$invalidate(1, showCreateForm = true);

		$$self.$capture_state = () => ({
			onMount,
			communityActions,
			communityStore,
			filteredLists,
			subscribedListIds,
			CommunityListCard,
			CommunityListDetail,
			CreateCommunityList,
			MySubscriptions,
			activeTab,
			showCreateForm,
			setActiveTab,
			handleSearch,
			handleSort,
			$subscribedListIds,
			$communityStore,
			$filteredLists
		});

		$$self.$inject_state = $$props => {
			if ('activeTab' in $$props) $$invalidate(0, activeTab = $$props.activeTab);
			if ('showCreateForm' in $$props) $$invalidate(1, showCreateForm = $$props.showCreateForm);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		return [
			activeTab,
			showCreateForm,
			$subscribedListIds,
			$communityStore,
			$filteredLists,
			setActiveTab,
			handleSearch,
			handleSort,
			click_handler,
			listCreated_handler,
			click_handler_1,
			click_handler_2,
			click_handler_3,
			click_handler_4
		];
	}

	class CommunityLists extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance$3, create_fragment$3, safe_not_equal, {});

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "CommunityLists",
				options,
				id: create_fragment$3.name
			});
		}
	}

	/* src/lib/components/UserProfile.svelte generated by Svelte v4.2.20 */
	const file$2 = "src/lib/components/UserProfile.svelte";

	// (220:2) {#if error}
	function create_if_block_14(ctx) {
		let div2;
		let div1;
		let svg;
		let path;
		let t0;
		let div0;
		let p;
		let t1;

		const block = {
			c: function create() {
				div2 = element("div");
				div1 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				div0 = element("div");
				p = element("p");
				t1 = text(/*error*/ ctx[2]);
				attr_dev(path, "fill-rule", "evenodd");
				attr_dev(path, "d", "M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z");
				attr_dev(path, "clip-rule", "evenodd");
				add_location(path, file$2, 250, 10, 6279);
				attr_dev(svg, "class", "h-5 w-5 text-red-400");
				attr_dev(svg, "viewBox", "0 0 20 20");
				attr_dev(svg, "fill", "currentColor");
				add_location(svg, file$2, 249, 8, 6194);
				attr_dev(p, "class", "text-sm text-red-800");
				add_location(p, file$2, 253, 10, 6600);
				attr_dev(div0, "class", "ml-3");
				add_location(div0, file$2, 252, 8, 6571);
				attr_dev(div1, "class", "flex");
				add_location(div1, file$2, 248, 6, 6167);
				attr_dev(div2, "class", "mb-6 bg-red-50 border border-red-200 rounded-md p-4");
				add_location(div2, file$2, 247, 4, 6095);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div2, anchor);
				append_dev(div2, div1);
				append_dev(div1, svg);
				append_dev(svg, path);
				append_dev(div1, t0);
				append_dev(div1, div0);
				append_dev(div0, p);
				append_dev(p, t1);
			},
			p: function update(ctx, dirty) {
				if (dirty[0] & /*error*/ 4) set_data_dev(t1, /*error*/ ctx[2]);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div2);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_14.name,
			type: "if",
			source: "(220:2) {#if error}",
			ctx
		});

		return block;
	}

	// (233:2) {#if success}
	function create_if_block_13(ctx) {
		let div2;
		let div1;
		let svg;
		let path;
		let t0;
		let div0;
		let p;
		let t1;

		const block = {
			c: function create() {
				div2 = element("div");
				div1 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				div0 = element("div");
				p = element("p");
				t1 = text(/*success*/ ctx[3]);
				attr_dev(path, "fill-rule", "evenodd");
				attr_dev(path, "d", "M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z");
				attr_dev(path, "clip-rule", "evenodd");
				add_location(path, file$2, 263, 10, 6902);
				attr_dev(svg, "class", "h-5 w-5 text-green-400");
				attr_dev(svg, "viewBox", "0 0 20 20");
				attr_dev(svg, "fill", "currentColor");
				add_location(svg, file$2, 262, 8, 6815);
				attr_dev(p, "class", "text-sm text-green-800");
				add_location(p, file$2, 266, 10, 7141);
				attr_dev(div0, "class", "ml-3");
				add_location(div0, file$2, 265, 8, 7112);
				attr_dev(div1, "class", "flex");
				add_location(div1, file$2, 261, 6, 6788);
				attr_dev(div2, "class", "mb-6 bg-green-50 border border-green-200 rounded-md p-4");
				add_location(div2, file$2, 260, 4, 6712);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div2, anchor);
				append_dev(div2, div1);
				append_dev(div1, svg);
				append_dev(svg, path);
				append_dev(div1, t0);
				append_dev(div1, div0);
				append_dev(div0, p);
				append_dev(p, t1);
			},
			p: function update(ctx, dirty) {
				if (dirty[0] & /*success*/ 8) set_data_dev(t1, /*success*/ ctx[3]);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div2);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_13.name,
			type: "if",
			source: "(233:2) {#if success}",
			ctx
		});

		return block;
	}

	// (251:20) 
	function create_if_block_3$1(ctx) {
		let div;
		let nav;
		let button0;
		let t0;
		let button0_class_value;
		let t1;
		let button1;
		let t2;
		let button1_class_value;
		let t3;
		let button2;
		let t4;
		let button2_class_value;
		let t5;
		let button3;
		let t6;
		let button3_class_value;
		let t7;
		let t8;
		let t9;
		let t10;
		let if_block3_anchor;
		let mounted;
		let dispose;
		let if_block0 = /*activeTab*/ ctx[4] === 'profile' && create_if_block_9(ctx);
		let if_block1 = /*activeTab*/ ctx[4] === 'security' && create_if_block_7$1(ctx);
		let if_block2 = /*activeTab*/ ctx[4] === 'preferences' && create_if_block_6$1(ctx);
		let if_block3 = /*activeTab*/ ctx[4] === 'data' && create_if_block_4$1(ctx);

		const block = {
			c: function create() {
				div = element("div");
				nav = element("nav");
				button0 = element("button");
				t0 = text("Profile");
				t1 = space();
				button1 = element("button");
				t2 = text("Security");
				t3 = space();
				button2 = element("button");
				t4 = text("Preferences");
				t5 = space();
				button3 = element("button");
				t6 = text("Data & Privacy");
				t7 = space();
				if (if_block0) if_block0.c();
				t8 = space();
				if (if_block1) if_block1.c();
				t9 = space();
				if (if_block2) if_block2.c();
				t10 = space();
				if (if_block3) if_block3.c();
				if_block3_anchor = empty();

				attr_dev(button0, "class", button0_class_value = "py-2 px-1 border-b-2 font-medium text-sm " + (/*activeTab*/ ctx[4] === 'profile'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'));

				add_location(button0, file$2, 281, 8, 7617);

				attr_dev(button1, "class", button1_class_value = "py-2 px-1 border-b-2 font-medium text-sm " + (/*activeTab*/ ctx[4] === 'security'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'));

				add_location(button1, file$2, 287, 8, 7930);

				attr_dev(button2, "class", button2_class_value = "py-2 px-1 border-b-2 font-medium text-sm " + (/*activeTab*/ ctx[4] === 'preferences'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'));

				add_location(button2, file$2, 293, 8, 8246);

				attr_dev(button3, "class", button3_class_value = "py-2 px-1 border-b-2 font-medium text-sm " + (/*activeTab*/ ctx[4] === 'data'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'));

				add_location(button3, file$2, 299, 8, 8571);
				attr_dev(nav, "class", "-mb-px flex space-x-8");
				add_location(nav, file$2, 280, 6, 7573);
				attr_dev(div, "class", "border-b border-gray-200 mb-6");
				add_location(div, file$2, 279, 4, 7523);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, nav);
				append_dev(nav, button0);
				append_dev(button0, t0);
				append_dev(nav, t1);
				append_dev(nav, button1);
				append_dev(button1, t2);
				append_dev(nav, t3);
				append_dev(nav, button2);
				append_dev(button2, t4);
				append_dev(nav, t5);
				append_dev(nav, button3);
				append_dev(button3, t6);
				insert_dev(target, t7, anchor);
				if (if_block0) if_block0.m(target, anchor);
				insert_dev(target, t8, anchor);
				if (if_block1) if_block1.m(target, anchor);
				insert_dev(target, t9, anchor);
				if (if_block2) if_block2.m(target, anchor);
				insert_dev(target, t10, anchor);
				if (if_block3) if_block3.m(target, anchor);
				insert_dev(target, if_block3_anchor, anchor);

				if (!mounted) {
					dispose = [
						listen_dev(button0, "click", /*click_handler*/ ctx[22], false, false, false, false),
						listen_dev(button1, "click", /*click_handler_1*/ ctx[23], false, false, false, false),
						listen_dev(button2, "click", /*click_handler_2*/ ctx[24], false, false, false, false),
						listen_dev(button3, "click", /*click_handler_3*/ ctx[25], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (dirty[0] & /*activeTab*/ 16 && button0_class_value !== (button0_class_value = "py-2 px-1 border-b-2 font-medium text-sm " + (/*activeTab*/ ctx[4] === 'profile'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'))) {
					attr_dev(button0, "class", button0_class_value);
				}

				if (dirty[0] & /*activeTab*/ 16 && button1_class_value !== (button1_class_value = "py-2 px-1 border-b-2 font-medium text-sm " + (/*activeTab*/ ctx[4] === 'security'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'))) {
					attr_dev(button1, "class", button1_class_value);
				}

				if (dirty[0] & /*activeTab*/ 16 && button2_class_value !== (button2_class_value = "py-2 px-1 border-b-2 font-medium text-sm " + (/*activeTab*/ ctx[4] === 'preferences'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'))) {
					attr_dev(button2, "class", button2_class_value);
				}

				if (dirty[0] & /*activeTab*/ 16 && button3_class_value !== (button3_class_value = "py-2 px-1 border-b-2 font-medium text-sm " + (/*activeTab*/ ctx[4] === 'data'
				? 'border-indigo-500 text-indigo-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'))) {
					attr_dev(button3, "class", button3_class_value);
				}

				if (/*activeTab*/ ctx[4] === 'profile') {
					if (if_block0) {
						if_block0.p(ctx, dirty);
					} else {
						if_block0 = create_if_block_9(ctx);
						if_block0.c();
						if_block0.m(t8.parentNode, t8);
					}
				} else if (if_block0) {
					if_block0.d(1);
					if_block0 = null;
				}

				if (/*activeTab*/ ctx[4] === 'security') {
					if (if_block1) {
						if_block1.p(ctx, dirty);
					} else {
						if_block1 = create_if_block_7$1(ctx);
						if_block1.c();
						if_block1.m(t9.parentNode, t9);
					}
				} else if (if_block1) {
					if_block1.d(1);
					if_block1 = null;
				}

				if (/*activeTab*/ ctx[4] === 'preferences') {
					if (if_block2) {
						if_block2.p(ctx, dirty);
					} else {
						if_block2 = create_if_block_6$1(ctx);
						if_block2.c();
						if_block2.m(t10.parentNode, t10);
					}
				} else if (if_block2) {
					if_block2.d(1);
					if_block2 = null;
				}

				if (/*activeTab*/ ctx[4] === 'data') {
					if (if_block3) {
						if_block3.p(ctx, dirty);
					} else {
						if_block3 = create_if_block_4$1(ctx);
						if_block3.c();
						if_block3.m(if_block3_anchor.parentNode, if_block3_anchor);
					}
				} else if (if_block3) {
					if_block3.d(1);
					if_block3 = null;
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
					detach_dev(t7);
					detach_dev(t8);
					detach_dev(t9);
					detach_dev(t10);
					detach_dev(if_block3_anchor);
				}

				if (if_block0) if_block0.d(detaching);
				if (if_block1) if_block1.d(detaching);
				if (if_block2) if_block2.d(detaching);
				if (if_block3) if_block3.d(detaching);
				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_3$1.name,
			type: "if",
			source: "(251:20) ",
			ctx
		});

		return block;
	}

	// (246:2) {#if isLoading && !profile}
	function create_if_block_2$1(ctx) {
		let div1;
		let div0;
		let t0;
		let p;

		const block = {
			c: function create() {
				div1 = element("div");
				div0 = element("div");
				t0 = space();
				p = element("p");
				p.textContent = "Loading profile...";
				attr_dev(div0, "class", "animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600 mx-auto");
				add_location(div0, file$2, 274, 6, 7309);
				attr_dev(p, "class", "mt-2 text-gray-600");
				add_location(p, file$2, 275, 6, 7406);
				attr_dev(div1, "class", "text-center py-12");
				add_location(div1, file$2, 273, 4, 7271);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div1, anchor);
				append_dev(div1, div0);
				append_dev(div1, t0);
				append_dev(div1, p);
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
			id: create_if_block_2$1.name,
			type: "if",
			source: "(246:2) {#if isLoading && !profile}",
			ctx
		});

		return block;
	}

	// (283:4) {#if activeTab === 'profile'}
	function create_if_block_9(ctx) {
		let div7;
		let div0;
		let h3;
		let t1;
		let p0;
		let t3;
		let div6;
		let div5;
		let div1;
		let label;
		let t5;
		let t6;
		let div4;
		let div2;
		let span0;
		let t8;
		let p1;
		let t9_value = formatDate(/*profile*/ ctx[0].created_at) + "";
		let t9;
		let t10;
		let div3;
		let span1;
		let t12;
		let p2;
		let t13_value = formatDate(/*profile*/ ctx[0].updated_at) + "";
		let t13;
		let t14;
		let t15;

		function select_block_type_1(ctx, dirty) {
			if (/*editingProfile*/ ctx[5]) return create_if_block_12;
			return create_else_block_2;
		}

		let current_block_type = select_block_type_1(ctx);
		let if_block0 = current_block_type(ctx);
		let if_block1 = /*profile*/ ctx[0].last_login && create_if_block_11(ctx);
		let if_block2 = /*editingProfile*/ ctx[5] && create_if_block_10(ctx);

		const block = {
			c: function create() {
				div7 = element("div");
				div0 = element("div");
				h3 = element("h3");
				h3.textContent = "Profile Information";
				t1 = space();
				p0 = element("p");
				p0.textContent = "Update your account's profile information and email address.";
				t3 = space();
				div6 = element("div");
				div5 = element("div");
				div1 = element("div");
				label = element("label");
				label.textContent = "Email";
				t5 = space();
				if_block0.c();
				t6 = space();
				div4 = element("div");
				div2 = element("div");
				span0 = element("span");
				span0.textContent = "Account Created";
				t8 = space();
				p1 = element("p");
				t9 = text(t9_value);
				t10 = space();
				div3 = element("div");
				span1 = element("span");
				span1.textContent = "Last Updated";
				t12 = space();
				p2 = element("p");
				t13 = text(t13_value);
				t14 = space();
				if (if_block1) if_block1.c();
				t15 = space();
				if (if_block2) if_block2.c();
				attr_dev(h3, "class", "text-lg font-medium text-gray-900");
				add_location(h3, file$2, 312, 10, 9075);
				attr_dev(p0, "class", "mt-1 text-sm text-gray-600");
				add_location(p0, file$2, 313, 10, 9156);
				attr_dev(div0, "class", "px-6 py-4 border-b border-gray-200");
				add_location(div0, file$2, 311, 8, 9016);
				attr_dev(label, "for", "email");
				attr_dev(label, "class", "block text-sm font-medium text-gray-700");
				add_location(label, file$2, 321, 14, 9423);
				add_location(div1, file$2, 320, 12, 9403);
				attr_dev(span0, "class", "block text-sm font-medium text-gray-700");
				add_location(span0, file$2, 348, 16, 10574);
				attr_dev(p1, "class", "mt-1 text-sm text-gray-900");
				add_location(p1, file$2, 349, 16, 10667);
				add_location(div2, file$2, 347, 14, 10552);
				attr_dev(span1, "class", "block text-sm font-medium text-gray-700");
				add_location(span1, file$2, 352, 16, 10799);
				attr_dev(p2, "class", "mt-1 text-sm text-gray-900");
				add_location(p2, file$2, 353, 16, 10889);
				add_location(div3, file$2, 351, 14, 10777);
				attr_dev(div4, "class", "grid grid-cols-1 gap-6 sm:grid-cols-2");
				add_location(div4, file$2, 346, 12, 10486);
				attr_dev(div5, "class", "space-y-6");
				add_location(div5, file$2, 318, 10, 9340);
				attr_dev(div6, "class", "px-6 py-4");
				add_location(div6, file$2, 317, 8, 9306);
				attr_dev(div7, "class", "bg-white shadow rounded-lg");
				add_location(div7, file$2, 310, 6, 8967);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div7, anchor);
				append_dev(div7, div0);
				append_dev(div0, h3);
				append_dev(div0, t1);
				append_dev(div0, p0);
				append_dev(div7, t3);
				append_dev(div7, div6);
				append_dev(div6, div5);
				append_dev(div5, div1);
				append_dev(div1, label);
				append_dev(div1, t5);
				if_block0.m(div1, null);
				append_dev(div5, t6);
				append_dev(div5, div4);
				append_dev(div4, div2);
				append_dev(div2, span0);
				append_dev(div2, t8);
				append_dev(div2, p1);
				append_dev(p1, t9);
				append_dev(div4, t10);
				append_dev(div4, div3);
				append_dev(div3, span1);
				append_dev(div3, t12);
				append_dev(div3, p2);
				append_dev(p2, t13);
				append_dev(div4, t14);
				if (if_block1) if_block1.m(div4, null);
				append_dev(div5, t15);
				if (if_block2) if_block2.m(div5, null);
			},
			p: function update(ctx, dirty) {
				if (current_block_type === (current_block_type = select_block_type_1(ctx)) && if_block0) {
					if_block0.p(ctx, dirty);
				} else {
					if_block0.d(1);
					if_block0 = current_block_type(ctx);

					if (if_block0) {
						if_block0.c();
						if_block0.m(div1, null);
					}
				}

				if (dirty[0] & /*profile*/ 1 && t9_value !== (t9_value = formatDate(/*profile*/ ctx[0].created_at) + "")) set_data_dev(t9, t9_value);
				if (dirty[0] & /*profile*/ 1 && t13_value !== (t13_value = formatDate(/*profile*/ ctx[0].updated_at) + "")) set_data_dev(t13, t13_value);

				if (/*profile*/ ctx[0].last_login) {
					if (if_block1) {
						if_block1.p(ctx, dirty);
					} else {
						if_block1 = create_if_block_11(ctx);
						if_block1.c();
						if_block1.m(div4, null);
					}
				} else if (if_block1) {
					if_block1.d(1);
					if_block1 = null;
				}

				if (/*editingProfile*/ ctx[5]) {
					if (if_block2) {
						if_block2.p(ctx, dirty);
					} else {
						if_block2 = create_if_block_10(ctx);
						if_block2.c();
						if_block2.m(div5, null);
					}
				} else if (if_block2) {
					if_block2.d(1);
					if_block2 = null;
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div7);
				}

				if_block0.d();
				if (if_block1) if_block1.d();
				if (if_block2) if_block2.d();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_9.name,
			type: "if",
			source: "(283:4) {#if activeTab === 'profile'}",
			ctx
		});

		return block;
	}

	// (306:14) {:else}
	function create_else_block_2(ctx) {
		let div;
		let span;
		let t0_value = /*profile*/ ctx[0].email + "";
		let t0;
		let t1;
		let button;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				div = element("div");
				span = element("span");
				t0 = text(t0_value);
				t1 = space();
				button = element("button");
				button.textContent = "Edit";
				attr_dev(span, "class", "text-sm text-gray-900");
				add_location(span, file$2, 334, 18, 10086);
				attr_dev(button, "class", "text-sm text-indigo-600 hover:text-indigo-500");
				add_location(button, file$2, 335, 18, 10163);
				attr_dev(div, "class", "mt-1 flex items-center justify-between");
				add_location(div, file$2, 333, 16, 10015);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, span);
				append_dev(span, t0);
				append_dev(div, t1);
				append_dev(div, button);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*click_handler_4*/ ctx[27], false, false, false, false);
					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (dirty[0] & /*profile*/ 1 && t0_value !== (t0_value = /*profile*/ ctx[0].email + "")) set_data_dev(t0, t0_value);
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
			id: create_else_block_2.name,
			type: "else",
			source: "(306:14) {:else}",
			ctx
		});

		return block;
	}

	// (296:14) {#if editingProfile}
	function create_if_block_12(ctx) {
		let div;
		let input;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				div = element("div");
				input = element("input");
				attr_dev(input, "type", "email");
				attr_dev(input, "id", "email");
				attr_dev(input, "class", "flex-1 min-w-0 block w-full px-3 py-2 rounded-md border-gray-300 focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm");
				attr_dev(input, "placeholder", "Enter your email");
				add_location(input, file$2, 324, 18, 9618);
				attr_dev(div, "class", "mt-1 flex rounded-md shadow-sm");
				add_location(div, file$2, 323, 16, 9555);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, input);
				set_input_value(input, /*editEmail*/ ctx[6]);

				if (!mounted) {
					dispose = listen_dev(input, "input", /*input_input_handler*/ ctx[26]);
					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (dirty[0] & /*editEmail*/ 64 && input.value !== /*editEmail*/ ctx[6]) {
					set_input_value(input, /*editEmail*/ ctx[6]);
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
			id: create_if_block_12.name,
			type: "if",
			source: "(296:14) {#if editingProfile}",
			ctx
		});

		return block;
	}

	// (329:14) {#if profile.last_login}
	function create_if_block_11(ctx) {
		let div;
		let span;
		let t1;
		let p;
		let t2_value = formatDate(/*profile*/ ctx[0].last_login) + "";
		let t2;

		const block = {
			c: function create() {
				div = element("div");
				span = element("span");
				span.textContent = "Last Login";
				t1 = space();
				p = element("p");
				t2 = text(t2_value);
				attr_dev(span, "class", "block text-sm font-medium text-gray-700");
				add_location(span, file$2, 357, 18, 11064);
				attr_dev(p, "class", "mt-1 text-sm text-gray-900");
				add_location(p, file$2, 358, 18, 11154);
				add_location(div, file$2, 356, 16, 11040);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, span);
				append_dev(div, t1);
				append_dev(div, p);
				append_dev(p, t2);
			},
			p: function update(ctx, dirty) {
				if (dirty[0] & /*profile*/ 1 && t2_value !== (t2_value = formatDate(/*profile*/ ctx[0].last_login) + "")) set_data_dev(t2, t2_value);
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_11.name,
			type: "if",
			source: "(329:14) {#if profile.last_login}",
			ctx
		});

		return block;
	}

	// (338:12) {#if editingProfile}
	function create_if_block_10(ctx) {
		let div;
		let button0;
		let t1;
		let button1;
		let t2_value = (/*isLoading*/ ctx[1] ? 'Saving...' : 'Save Changes') + "";
		let t2;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				div = element("div");
				button0 = element("button");
				button0.textContent = "Cancel";
				t1 = space();
				button1 = element("button");
				t2 = text(t2_value);
				attr_dev(button0, "class", "px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50");
				add_location(button0, file$2, 366, 16, 11432);
				button1.disabled = /*isLoading*/ ctx[1];
				attr_dev(button1, "class", "px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50");
				add_location(button1, file$2, 375, 16, 11801);
				attr_dev(div, "class", "flex justify-end space-x-3");
				add_location(div, file$2, 365, 14, 11375);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				append_dev(div, button0);
				append_dev(div, t1);
				append_dev(div, button1);
				append_dev(button1, t2);

				if (!mounted) {
					dispose = [
						listen_dev(button0, "click", /*click_handler_5*/ ctx[28], false, false, false, false),
						listen_dev(button1, "click", /*updateProfile*/ ctx[15], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (dirty[0] & /*isLoading*/ 2 && t2_value !== (t2_value = (/*isLoading*/ ctx[1] ? 'Saving...' : 'Save Changes') + "")) set_data_dev(t2, t2_value);

				if (dirty[0] & /*isLoading*/ 2) {
					prop_dev(button1, "disabled", /*isLoading*/ ctx[1]);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}

				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_10.name,
			type: "if",
			source: "(338:12) {#if editingProfile}",
			ctx
		});

		return block;
	}

	// (364:4) {#if activeTab === 'security'}
	function create_if_block_7$1(ctx) {
		let div6;
		let div5;
		let div0;
		let h3;
		let t1;
		let p0;
		let t3;
		let div4;
		let div3;
		let div1;
		let p1;
		let t4;
		let t5_value = (/*profile*/ ctx[0].totp_enabled ? 'Enabled' : 'Disabled') + "";
		let t5;
		let t6;
		let p2;

		let t7_value = (/*profile*/ ctx[0].totp_enabled
		? 'Your account is protected with 2FA'
		: 'Enable 2FA to secure your account') + "";

		let t7;
		let t8;
		let div2;

		function select_block_type_2(ctx, dirty) {
			if (/*profile*/ ctx[0].totp_enabled) return create_if_block_8;
			return create_else_block_1$1;
		}

		let current_block_type = select_block_type_2(ctx);
		let if_block = current_block_type(ctx);

		const block = {
			c: function create() {
				div6 = element("div");
				div5 = element("div");
				div0 = element("div");
				h3 = element("h3");
				h3.textContent = "Two-Factor Authentication";
				t1 = space();
				p0 = element("p");
				p0.textContent = "Add an extra layer of security to your account with 2FA.";
				t3 = space();
				div4 = element("div");
				div3 = element("div");
				div1 = element("div");
				p1 = element("p");
				t4 = text("Status: ");
				t5 = text(t5_value);
				t6 = space();
				p2 = element("p");
				t7 = text(t7_value);
				t8 = space();
				div2 = element("div");
				if_block.c();
				attr_dev(h3, "class", "text-lg font-medium text-gray-900");
				add_location(h3, file$2, 395, 12, 12513);
				attr_dev(p0, "class", "mt-1 text-sm text-gray-600");
				add_location(p0, file$2, 396, 12, 12602);
				attr_dev(div0, "class", "px-6 py-4 border-b border-gray-200");
				add_location(div0, file$2, 394, 10, 12452);
				attr_dev(p1, "class", "text-sm font-medium text-gray-900");
				add_location(p1, file$2, 403, 16, 12876);
				attr_dev(p2, "class", "text-sm text-gray-600");
				add_location(p2, file$2, 406, 16, 13033);
				add_location(div1, file$2, 402, 14, 12854);
				add_location(div2, file$2, 412, 14, 13283);
				attr_dev(div3, "class", "flex items-center justify-between");
				add_location(div3, file$2, 401, 12, 12792);
				attr_dev(div4, "class", "px-6 py-4");
				add_location(div4, file$2, 400, 10, 12756);
				attr_dev(div5, "class", "bg-white shadow rounded-lg");
				add_location(div5, file$2, 393, 8, 12401);
				attr_dev(div6, "class", "space-y-6");
				add_location(div6, file$2, 391, 6, 12326);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div6, anchor);
				append_dev(div6, div5);
				append_dev(div5, div0);
				append_dev(div0, h3);
				append_dev(div0, t1);
				append_dev(div0, p0);
				append_dev(div5, t3);
				append_dev(div5, div4);
				append_dev(div4, div3);
				append_dev(div3, div1);
				append_dev(div1, p1);
				append_dev(p1, t4);
				append_dev(p1, t5);
				append_dev(div1, t6);
				append_dev(div1, p2);
				append_dev(p2, t7);
				append_dev(div3, t8);
				append_dev(div3, div2);
				if_block.m(div2, null);
			},
			p: function update(ctx, dirty) {
				if (dirty[0] & /*profile*/ 1 && t5_value !== (t5_value = (/*profile*/ ctx[0].totp_enabled ? 'Enabled' : 'Disabled') + "")) set_data_dev(t5, t5_value);

				if (dirty[0] & /*profile*/ 1 && t7_value !== (t7_value = (/*profile*/ ctx[0].totp_enabled
				? 'Your account is protected with 2FA'
				: 'Enable 2FA to secure your account') + "")) set_data_dev(t7, t7_value);

				if (current_block_type === (current_block_type = select_block_type_2(ctx)) && if_block) {
					if_block.p(ctx, dirty);
				} else {
					if_block.d(1);
					if_block = current_block_type(ctx);

					if (if_block) {
						if_block.c();
						if_block.m(div2, null);
					}
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div6);
				}

				if_block.d();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_7$1.name,
			type: "if",
			source: "(364:4) {#if activeTab === 'security'}",
			ctx
		});

		return block;
	}

	// (394:16) {:else}
	function create_else_block_1$1(ctx) {
		let button;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				button = element("button");
				button.textContent = "Enable 2FA";
				attr_dev(button, "class", "px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700");
				add_location(button, file$2, 421, 18, 13659);
			},
			m: function mount(target, anchor) {
				insert_dev(target, button, anchor);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*click_handler_7*/ ctx[30], false, false, false, false);
					mounted = true;
				}
			},
			p: noop,
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
			source: "(394:16) {:else}",
			ctx
		});

		return block;
	}

	// (387:16) {#if profile.totp_enabled}
	function create_if_block_8(ctx) {
		let button;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				button = element("button");
				button.textContent = "Disable 2FA";
				attr_dev(button, "class", "px-4 py-2 border border-red-300 rounded-md text-sm font-medium text-red-700 hover:bg-red-50");
				add_location(button, file$2, 414, 18, 13350);
			},
			m: function mount(target, anchor) {
				insert_dev(target, button, anchor);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*click_handler_6*/ ctx[29], false, false, false, false);
					mounted = true;
				}
			},
			p: noop,
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
			id: create_if_block_8.name,
			type: "if",
			source: "(387:16) {#if profile.totp_enabled}",
			ctx
		});

		return block;
	}

	// (410:4) {#if activeTab === 'preferences'}
	function create_if_block_6$1(ctx) {
		let div10;
		let div0;
		let h3;
		let t1;
		let p0;
		let t3;
		let div9;
		let div8;
		let div3;
		let div1;
		let span0;
		let t5;
		let p1;
		let t7;
		let label0;
		let input0;
		let t8;
		let div2;
		let t9;
		let div6;
		let div4;
		let span1;
		let t11;
		let p2;
		let t13;
		let label1;
		let input1;
		let t14;
		let div5;
		let t15;
		let div7;
		let button;
		let t16_value = (/*isLoading*/ ctx[1] ? 'Saving...' : 'Save Preferences') + "";
		let t16;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				div10 = element("div");
				div0 = element("div");
				h3 = element("h3");
				h3.textContent = "Preferences";
				t1 = space();
				p0 = element("p");
				p0.textContent = "Customize your experience and notification settings.";
				t3 = space();
				div9 = element("div");
				div8 = element("div");
				div3 = element("div");
				div1 = element("div");
				span0 = element("span");
				span0.textContent = "Email Notifications";
				t5 = space();
				p1 = element("p");
				p1.textContent = "Receive email updates about your account activity";
				t7 = space();
				label0 = element("label");
				input0 = element("input");
				t8 = space();
				div2 = element("div");
				t9 = space();
				div6 = element("div");
				div4 = element("div");
				span1 = element("span");
				span1.textContent = "Privacy Mode";
				t11 = space();
				p2 = element("p");
				p2.textContent = "Hide your activity from other users";
				t13 = space();
				label1 = element("label");
				input1 = element("input");
				t14 = space();
				div5 = element("div");
				t15 = space();
				div7 = element("div");
				button = element("button");
				t16 = text(t16_value);
				attr_dev(h3, "class", "text-lg font-medium text-gray-900");
				add_location(h3, file$2, 439, 10, 14252);
				attr_dev(p0, "class", "mt-1 text-sm text-gray-600");
				add_location(p0, file$2, 440, 10, 14325);
				attr_dev(div0, "class", "px-6 py-4 border-b border-gray-200");
				add_location(div0, file$2, 438, 8, 14193);
				attr_dev(span0, "class", "text-sm font-medium text-gray-900");
				add_location(span0, file$2, 449, 16, 14662);
				attr_dev(p1, "class", "text-sm text-gray-600");
				add_location(p1, file$2, 450, 16, 14753);
				add_location(div1, file$2, 448, 14, 14640);
				attr_dev(input0, "type", "checkbox");
				attr_dev(input0, "class", "sr-only peer");
				add_location(input0, file$2, 453, 16, 14956);
				attr_dev(div2, "class", "w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-indigo-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-600");
				add_location(div2, file$2, 458, 16, 15137);
				attr_dev(label0, "class", "relative inline-flex items-center cursor-pointer");
				add_location(label0, file$2, 452, 14, 14875);
				attr_dev(div3, "class", "flex items-center justify-between");
				add_location(div3, file$2, 447, 12, 14578);
				attr_dev(span1, "class", "text-sm font-medium text-gray-900");
				add_location(span1, file$2, 465, 16, 15710);
				attr_dev(p2, "class", "text-sm text-gray-600");
				add_location(p2, file$2, 466, 16, 15794);
				add_location(div4, file$2, 464, 14, 15688);
				attr_dev(input1, "type", "checkbox");
				attr_dev(input1, "class", "sr-only peer");
				add_location(input1, file$2, 469, 16, 15983);
				attr_dev(div5, "class", "w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-indigo-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-indigo-600");
				add_location(div5, file$2, 474, 16, 16157);
				attr_dev(label1, "class", "relative inline-flex items-center cursor-pointer");
				add_location(label1, file$2, 468, 14, 15902);
				attr_dev(div6, "class", "flex items-center justify-between");
				add_location(div6, file$2, 463, 12, 15626);
				button.disabled = /*isLoading*/ ctx[1];
				attr_dev(button, "class", "px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50");
				add_location(button, file$2, 480, 14, 16690);
				attr_dev(div7, "class", "flex justify-end");
				add_location(div7, file$2, 479, 12, 16645);
				attr_dev(div8, "class", "space-y-6");
				add_location(div8, file$2, 445, 10, 14501);
				attr_dev(div9, "class", "px-6 py-4");
				add_location(div9, file$2, 444, 8, 14467);
				attr_dev(div10, "class", "bg-white shadow rounded-lg");
				add_location(div10, file$2, 437, 6, 14144);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div10, anchor);
				append_dev(div10, div0);
				append_dev(div0, h3);
				append_dev(div0, t1);
				append_dev(div0, p0);
				append_dev(div10, t3);
				append_dev(div10, div9);
				append_dev(div9, div8);
				append_dev(div8, div3);
				append_dev(div3, div1);
				append_dev(div1, span0);
				append_dev(div1, t5);
				append_dev(div1, p1);
				append_dev(div3, t7);
				append_dev(div3, label0);
				append_dev(label0, input0);
				input0.checked = /*editSettings*/ ctx[7].email_notifications;
				append_dev(label0, t8);
				append_dev(label0, div2);
				append_dev(div8, t9);
				append_dev(div8, div6);
				append_dev(div6, div4);
				append_dev(div4, span1);
				append_dev(div4, t11);
				append_dev(div4, p2);
				append_dev(div6, t13);
				append_dev(div6, label1);
				append_dev(label1, input1);
				input1.checked = /*editSettings*/ ctx[7].privacy_mode;
				append_dev(label1, t14);
				append_dev(label1, div5);
				append_dev(div8, t15);
				append_dev(div8, div7);
				append_dev(div7, button);
				append_dev(button, t16);

				if (!mounted) {
					dispose = [
						listen_dev(input0, "change", /*input0_change_handler*/ ctx[31]),
						listen_dev(input1, "change", /*input1_change_handler*/ ctx[32]),
						listen_dev(button, "click", /*updateSettings*/ ctx[16], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (dirty[0] & /*editSettings*/ 128) {
					input0.checked = /*editSettings*/ ctx[7].email_notifications;
				}

				if (dirty[0] & /*editSettings*/ 128) {
					input1.checked = /*editSettings*/ ctx[7].privacy_mode;
				}

				if (dirty[0] & /*isLoading*/ 2 && t16_value !== (t16_value = (/*isLoading*/ ctx[1] ? 'Saving...' : 'Save Preferences') + "")) set_data_dev(t16, t16_value);

				if (dirty[0] & /*isLoading*/ 2) {
					prop_dev(button, "disabled", /*isLoading*/ ctx[1]);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div10);
				}

				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_6$1.name,
			type: "if",
			source: "(410:4) {#if activeTab === 'preferences'}",
			ctx
		});

		return block;
	}

	// (468:4) {#if activeTab === 'data'}
	function create_if_block_4$1(ctx) {
		let div6;
		let div2;
		let div0;
		let h30;
		let t1;
		let p0;
		let t3;
		let div1;
		let button;

		let t4_value = (/*exportLoading*/ ctx[10]
		? 'Exporting...'
		: 'Export My Data') + "";

		let t4;
		let t5;
		let div5;
		let div3;
		let h31;
		let t7;
		let p1;
		let t9;
		let div4;
		let mounted;
		let dispose;

		function select_block_type_3(ctx, dirty) {
			if (!/*showDeleteConfirm*/ ctx[11]) return create_if_block_5$1;
			return create_else_block$2;
		}

		let current_block_type = select_block_type_3(ctx);
		let if_block = current_block_type(ctx);

		const block = {
			c: function create() {
				div6 = element("div");
				div2 = element("div");
				div0 = element("div");
				h30 = element("h3");
				h30.textContent = "Data Export";
				t1 = space();
				p0 = element("p");
				p0.textContent = "Download a copy of all your data for your records.";
				t3 = space();
				div1 = element("div");
				button = element("button");
				t4 = text(t4_value);
				t5 = space();
				div5 = element("div");
				div3 = element("div");
				h31 = element("h3");
				h31.textContent = "Delete Account";
				t7 = space();
				p1 = element("p");
				p1.textContent = "Permanently delete your account and all associated data.";
				t9 = space();
				div4 = element("div");
				if_block.c();
				attr_dev(h30, "class", "text-lg font-medium text-gray-900");
				add_location(h30, file$2, 499, 12, 17363);
				attr_dev(p0, "class", "mt-1 text-sm text-gray-600");
				add_location(p0, file$2, 500, 12, 17438);
				attr_dev(div0, "class", "px-6 py-4 border-b border-gray-200");
				add_location(div0, file$2, 498, 10, 17302);
				button.disabled = /*exportLoading*/ ctx[10];
				attr_dev(button, "class", "px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50");
				add_location(button, file$2, 505, 12, 17622);
				attr_dev(div1, "class", "px-6 py-4");
				add_location(div1, file$2, 504, 10, 17586);
				attr_dev(div2, "class", "bg-white shadow rounded-lg");
				add_location(div2, file$2, 497, 8, 17251);
				attr_dev(h31, "class", "text-lg font-medium text-red-900");
				add_location(h31, file$2, 518, 12, 18172);
				attr_dev(p1, "class", "mt-1 text-sm text-red-600");
				add_location(p1, file$2, 519, 12, 18249);
				attr_dev(div3, "class", "px-6 py-4 border-b border-red-200");
				add_location(div3, file$2, 517, 10, 18112);
				attr_dev(div4, "class", "px-6 py-4");
				add_location(div4, file$2, 523, 10, 18402);
				attr_dev(div5, "class", "bg-white shadow rounded-lg border-red-200");
				add_location(div5, file$2, 516, 8, 18046);
				attr_dev(div6, "class", "space-y-6");
				add_location(div6, file$2, 495, 6, 17190);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div6, anchor);
				append_dev(div6, div2);
				append_dev(div2, div0);
				append_dev(div0, h30);
				append_dev(div0, t1);
				append_dev(div0, p0);
				append_dev(div2, t3);
				append_dev(div2, div1);
				append_dev(div1, button);
				append_dev(button, t4);
				append_dev(div6, t5);
				append_dev(div6, div5);
				append_dev(div5, div3);
				append_dev(div3, h31);
				append_dev(div3, t7);
				append_dev(div3, p1);
				append_dev(div5, t9);
				append_dev(div5, div4);
				if_block.m(div4, null);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*exportData*/ ctx[17], false, false, false, false);
					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (dirty[0] & /*exportLoading*/ 1024 && t4_value !== (t4_value = (/*exportLoading*/ ctx[10]
				? 'Exporting...'
				: 'Export My Data') + "")) set_data_dev(t4, t4_value);

				if (dirty[0] & /*exportLoading*/ 1024) {
					prop_dev(button, "disabled", /*exportLoading*/ ctx[10]);
				}

				if (current_block_type === (current_block_type = select_block_type_3(ctx)) && if_block) {
					if_block.p(ctx, dirty);
				} else {
					if_block.d(1);
					if_block = current_block_type(ctx);

					if (if_block) {
						if_block.c();
						if_block.m(div4, null);
					}
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div6);
				}

				if_block.d();
				mounted = false;
				dispose();
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_4$1.name,
			type: "if",
			source: "(468:4) {#if activeTab === 'data'}",
			ctx
		});

		return block;
	}

	// (505:12) {:else}
	function create_else_block$2(ctx) {
		let div6;
		let div2;
		let div1;
		let svg;
		let path;
		let t0;
		let div0;
		let h3;
		let t2;
		let p;
		let t4;
		let div3;
		let label0;
		let t6;
		let input;
		let input_placeholder_value;
		let t7;
		let div4;
		let label1;
		let t9;
		let textarea;
		let t10;
		let div5;
		let button0;
		let t12;
		let button1;

		let t13_value = (/*deleteLoading*/ ctx[14]
		? 'Deleting...'
		: 'Delete Account') + "";

		let t13;
		let button1_disabled_value;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				div6 = element("div");
				div2 = element("div");
				div1 = element("div");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				div0 = element("div");
				h3 = element("h3");
				h3.textContent = "This action cannot be undone";
				t2 = space();
				p = element("p");
				p.textContent = "This will permanently delete your account, DNP lists, and all associated data.";
				t4 = space();
				div3 = element("div");
				label0 = element("label");
				label0.textContent = "Confirm your email address";
				t6 = space();
				input = element("input");
				t7 = space();
				div4 = element("div");
				label1 = element("label");
				label1.textContent = "Reason for deletion (optional)";
				t9 = space();
				textarea = element("textarea");
				t10 = space();
				div5 = element("div");
				button0 = element("button");
				button0.textContent = "Cancel";
				t12 = space();
				button1 = element("button");
				t13 = text(t13_value);
				attr_dev(path, "fill-rule", "evenodd");
				attr_dev(path, "d", "M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z");
				attr_dev(path, "clip-rule", "evenodd");
				add_location(path, file$2, 536, 22, 19019);
				attr_dev(svg, "class", "h-5 w-5 text-red-400");
				attr_dev(svg, "viewBox", "0 0 20 20");
				attr_dev(svg, "fill", "currentColor");
				add_location(svg, file$2, 535, 20, 18922);
				attr_dev(h3, "class", "text-sm font-medium text-red-800");
				add_location(h3, file$2, 539, 22, 19370);
				attr_dev(p, "class", "mt-2 text-sm text-red-700");
				add_location(p, file$2, 542, 22, 19519);
				attr_dev(div0, "class", "ml-3");
				add_location(div0, file$2, 538, 20, 19329);
				attr_dev(div1, "class", "flex");
				add_location(div1, file$2, 534, 18, 18883);
				attr_dev(div2, "class", "bg-red-50 border border-red-200 rounded-md p-4");
				add_location(div2, file$2, 533, 16, 18804);
				attr_dev(label0, "for", "confirm-email");
				attr_dev(label0, "class", "block text-sm font-medium text-gray-700");
				add_location(label0, file$2, 550, 18, 19803);
				attr_dev(input, "type", "email");
				attr_dev(input, "id", "confirm-email");
				attr_dev(input, "placeholder", input_placeholder_value = /*profile*/ ctx[0].email);
				attr_dev(input, "class", "mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-red-500 focus:border-red-500 sm:text-sm");
				add_location(input, file$2, 553, 18, 19971);
				add_location(div3, file$2, 549, 16, 19779);
				attr_dev(label1, "for", "delete-reason");
				attr_dev(label1, "class", "block text-sm font-medium text-gray-700");
				add_location(label1, file$2, 563, 18, 20386);
				attr_dev(textarea, "id", "delete-reason");
				attr_dev(textarea, "rows", "3");
				attr_dev(textarea, "class", "mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-red-500 focus:border-red-500 sm:text-sm");
				attr_dev(textarea, "placeholder", "Help us improve by telling us why you're leaving...");
				add_location(textarea, file$2, 566, 18, 20558);
				add_location(div4, file$2, 562, 16, 20362);
				attr_dev(button0, "class", "px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50");
				add_location(button0, file$2, 576, 18, 21049);
				button1.disabled = button1_disabled_value = /*deleteLoading*/ ctx[14] || /*deleteConfirmEmail*/ ctx[12] !== /*profile*/ ctx[0].email;
				attr_dev(button1, "class", "px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-red-600 hover:bg-red-700 disabled:opacity-50");
				add_location(button1, file$2, 586, 18, 21471);
				attr_dev(div5, "class", "flex justify-end space-x-3");
				add_location(div5, file$2, 575, 16, 20990);
				attr_dev(div6, "class", "space-y-4");
				add_location(div6, file$2, 532, 14, 18764);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div6, anchor);
				append_dev(div6, div2);
				append_dev(div2, div1);
				append_dev(div1, svg);
				append_dev(svg, path);
				append_dev(div1, t0);
				append_dev(div1, div0);
				append_dev(div0, h3);
				append_dev(div0, t2);
				append_dev(div0, p);
				append_dev(div6, t4);
				append_dev(div6, div3);
				append_dev(div3, label0);
				append_dev(div3, t6);
				append_dev(div3, input);
				set_input_value(input, /*deleteConfirmEmail*/ ctx[12]);
				append_dev(div6, t7);
				append_dev(div6, div4);
				append_dev(div4, label1);
				append_dev(div4, t9);
				append_dev(div4, textarea);
				set_input_value(textarea, /*deleteReason*/ ctx[13]);
				append_dev(div6, t10);
				append_dev(div6, div5);
				append_dev(div5, button0);
				append_dev(div5, t12);
				append_dev(div5, button1);
				append_dev(button1, t13);

				if (!mounted) {
					dispose = [
						listen_dev(input, "input", /*input_input_handler_1*/ ctx[34]),
						listen_dev(textarea, "input", /*textarea_input_handler*/ ctx[35]),
						listen_dev(button0, "click", /*click_handler_9*/ ctx[36], false, false, false, false),
						listen_dev(button1, "click", /*deleteAccount*/ ctx[18], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (dirty[0] & /*profile*/ 1 && input_placeholder_value !== (input_placeholder_value = /*profile*/ ctx[0].email)) {
					attr_dev(input, "placeholder", input_placeholder_value);
				}

				if (dirty[0] & /*deleteConfirmEmail*/ 4096 && input.value !== /*deleteConfirmEmail*/ ctx[12]) {
					set_input_value(input, /*deleteConfirmEmail*/ ctx[12]);
				}

				if (dirty[0] & /*deleteReason*/ 8192) {
					set_input_value(textarea, /*deleteReason*/ ctx[13]);
				}

				if (dirty[0] & /*deleteLoading*/ 16384 && t13_value !== (t13_value = (/*deleteLoading*/ ctx[14]
				? 'Deleting...'
				: 'Delete Account') + "")) set_data_dev(t13, t13_value);

				if (dirty[0] & /*deleteLoading, deleteConfirmEmail, profile*/ 20481 && button1_disabled_value !== (button1_disabled_value = /*deleteLoading*/ ctx[14] || /*deleteConfirmEmail*/ ctx[12] !== /*profile*/ ctx[0].email)) {
					prop_dev(button1, "disabled", button1_disabled_value);
				}
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div6);
				}

				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_else_block$2.name,
			type: "else",
			source: "(505:12) {:else}",
			ctx
		});

		return block;
	}

	// (498:12) {#if !showDeleteConfirm}
	function create_if_block_5$1(ctx) {
		let button;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				button = element("button");
				button.textContent = "Delete Account";
				attr_dev(button, "class", "px-4 py-2 border border-red-300 rounded-md text-sm font-medium text-red-700 hover:bg-red-50");
				add_location(button, file$2, 525, 14, 18477);
			},
			m: function mount(target, anchor) {
				insert_dev(target, button, anchor);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*click_handler_8*/ ctx[33], false, false, false, false);
					mounted = true;
				}
			},
			p: noop,
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
			id: create_if_block_5$1.name,
			type: "if",
			source: "(498:12) {#if !showDeleteConfirm}",
			ctx
		});

		return block;
	}

	// (578:0) {#if show2FASetup}
	function create_if_block_1$2(ctx) {
		let twofactorsetup;
		let current;
		twofactorsetup = new TwoFactorSetup({ $$inline: true });
		twofactorsetup.$on("complete", /*handle2FASetupComplete*/ ctx[19]);
		twofactorsetup.$on("cancel", /*handle2FASetupCancel*/ ctx[20]);

		const block = {
			c: function create() {
				create_component(twofactorsetup.$$.fragment);
			},
			m: function mount(target, anchor) {
				mount_component(twofactorsetup, target, anchor);
				current = true;
			},
			p: noop,
			i: function intro(local) {
				if (current) return;
				transition_in(twofactorsetup.$$.fragment, local);
				current = true;
			},
			o: function outro(local) {
				transition_out(twofactorsetup.$$.fragment, local);
				current = false;
			},
			d: function destroy(detaching) {
				destroy_component(twofactorsetup, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_1$2.name,
			type: "if",
			source: "(578:0) {#if show2FASetup}",
			ctx
		});

		return block;
	}

	// (586:0) {#if show2FADisable}
	function create_if_block$2(ctx) {
		let div4;
		let div3;
		let div2;
		let h3;
		let t1;
		let p;
		let t3;
		let div0;
		let input;
		let t4;
		let div1;
		let button;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				div4 = element("div");
				div3 = element("div");
				div2 = element("div");
				h3 = element("h3");
				h3.textContent = "Disable Two-Factor Authentication";
				t1 = space();
				p = element("p");
				p.textContent = "Enter your 2FA code to disable two-factor authentication";
				t3 = space();
				div0 = element("div");
				input = element("input");
				t4 = space();
				div1 = element("div");
				button = element("button");
				button.textContent = "Cancel";
				attr_dev(h3, "class", "text-lg font-medium text-gray-900 mb-4");
				add_location(h3, file$2, 616, 8, 22449);
				attr_dev(p, "class", "text-sm text-gray-600 mb-6");
				add_location(p, file$2, 619, 8, 22567);
				attr_dev(input, "type", "text");
				attr_dev(input, "placeholder", "Enter 6-digit code");
				attr_dev(input, "maxlength", "6");
				attr_dev(input, "class", "w-full px-3 py-2 border border-gray-300 rounded-md text-center text-lg tracking-widest");
				add_location(input, file$2, 624, 10, 22732);
				attr_dev(div0, "class", "mb-4");
				add_location(div0, file$2, 623, 8, 22703);
				attr_dev(button, "class", "px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50");
				add_location(button, file$2, 634, 10, 23079);
				attr_dev(div1, "class", "flex justify-center space-x-3");
				add_location(div1, file$2, 633, 8, 23025);
				attr_dev(div2, "class", "mt-3 text-center");
				add_location(div2, file$2, 615, 6, 22410);
				attr_dev(div3, "class", "relative top-20 mx-auto p-5 border w-96 shadow-lg rounded-md bg-white");
				add_location(div3, file$2, 614, 4, 22320);
				attr_dev(div4, "class", "fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50");
				add_location(div4, file$2, 613, 2, 22227);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div4, anchor);
				append_dev(div4, div3);
				append_dev(div3, div2);
				append_dev(div2, h3);
				append_dev(div2, t1);
				append_dev(div2, p);
				append_dev(div2, t3);
				append_dev(div2, div0);
				append_dev(div0, input);
				append_dev(div2, t4);
				append_dev(div2, div1);
				append_dev(div1, button);

				if (!mounted) {
					dispose = [
						listen_dev(input, "input", /*handleTotpInput*/ ctx[21], false, false, false, false),
						listen_dev(button, "click", /*click_handler_10*/ ctx[37], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div4);
				}

				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block$2.name,
			type: "if",
			source: "(586:0) {#if show2FADisable}",
			ctx
		});

		return block;
	}

	function create_fragment$2(ctx) {
		let div1;
		let div0;
		let h1;
		let t1;
		let p;
		let t3;
		let t4;
		let t5;
		let t6;
		let t7;
		let if_block4_anchor;
		let current;
		let if_block0 = /*error*/ ctx[2] && create_if_block_14(ctx);
		let if_block1 = /*success*/ ctx[3] && create_if_block_13(ctx);

		function select_block_type(ctx, dirty) {
			if (/*isLoading*/ ctx[1] && !/*profile*/ ctx[0]) return create_if_block_2$1;
			if (/*profile*/ ctx[0]) return create_if_block_3$1;
		}

		let current_block_type = select_block_type(ctx);
		let if_block2 = current_block_type && current_block_type(ctx);
		let if_block3 = /*show2FASetup*/ ctx[8] && create_if_block_1$2(ctx);
		let if_block4 = /*show2FADisable*/ ctx[9] && create_if_block$2(ctx);

		const block = {
			c: function create() {
				div1 = element("div");
				div0 = element("div");
				h1 = element("h1");
				h1.textContent = "Account Settings";
				t1 = space();
				p = element("p");
				p.textContent = "Manage your account information, security settings, and preferences.";
				t3 = space();
				if (if_block0) if_block0.c();
				t4 = space();
				if (if_block1) if_block1.c();
				t5 = space();
				if (if_block2) if_block2.c();
				t6 = space();
				if (if_block3) if_block3.c();
				t7 = space();
				if (if_block4) if_block4.c();
				if_block4_anchor = empty();
				attr_dev(h1, "class", "text-2xl font-bold text-gray-900");
				add_location(h1, file$2, 239, 4, 5839);
				attr_dev(p, "class", "mt-1 text-sm text-gray-600");
				add_location(p, file$2, 240, 4, 5910);
				attr_dev(div0, "class", "mb-8");
				add_location(div0, file$2, 238, 2, 5816);
				attr_dev(div1, "class", "max-w-4xl mx-auto py-6 px-4 sm:px-6 lg:px-8");
				add_location(div1, file$2, 236, 0, 5738);
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				insert_dev(target, div1, anchor);
				append_dev(div1, div0);
				append_dev(div0, h1);
				append_dev(div0, t1);
				append_dev(div0, p);
				append_dev(div1, t3);
				if (if_block0) if_block0.m(div1, null);
				append_dev(div1, t4);
				if (if_block1) if_block1.m(div1, null);
				append_dev(div1, t5);
				if (if_block2) if_block2.m(div1, null);
				insert_dev(target, t6, anchor);
				if (if_block3) if_block3.m(target, anchor);
				insert_dev(target, t7, anchor);
				if (if_block4) if_block4.m(target, anchor);
				insert_dev(target, if_block4_anchor, anchor);
				current = true;
			},
			p: function update(ctx, dirty) {
				if (/*error*/ ctx[2]) {
					if (if_block0) {
						if_block0.p(ctx, dirty);
					} else {
						if_block0 = create_if_block_14(ctx);
						if_block0.c();
						if_block0.m(div1, t4);
					}
				} else if (if_block0) {
					if_block0.d(1);
					if_block0 = null;
				}

				if (/*success*/ ctx[3]) {
					if (if_block1) {
						if_block1.p(ctx, dirty);
					} else {
						if_block1 = create_if_block_13(ctx);
						if_block1.c();
						if_block1.m(div1, t5);
					}
				} else if (if_block1) {
					if_block1.d(1);
					if_block1 = null;
				}

				if (current_block_type === (current_block_type = select_block_type(ctx)) && if_block2) {
					if_block2.p(ctx, dirty);
				} else {
					if (if_block2) if_block2.d(1);
					if_block2 = current_block_type && current_block_type(ctx);

					if (if_block2) {
						if_block2.c();
						if_block2.m(div1, null);
					}
				}

				if (/*show2FASetup*/ ctx[8]) {
					if (if_block3) {
						if_block3.p(ctx, dirty);

						if (dirty[0] & /*show2FASetup*/ 256) {
							transition_in(if_block3, 1);
						}
					} else {
						if_block3 = create_if_block_1$2(ctx);
						if_block3.c();
						transition_in(if_block3, 1);
						if_block3.m(t7.parentNode, t7);
					}
				} else if (if_block3) {
					group_outros();

					transition_out(if_block3, 1, 1, () => {
						if_block3 = null;
					});

					check_outros();
				}

				if (/*show2FADisable*/ ctx[9]) {
					if (if_block4) {
						if_block4.p(ctx, dirty);
					} else {
						if_block4 = create_if_block$2(ctx);
						if_block4.c();
						if_block4.m(if_block4_anchor.parentNode, if_block4_anchor);
					}
				} else if (if_block4) {
					if_block4.d(1);
					if_block4 = null;
				}
			},
			i: function intro(local) {
				if (current) return;
				transition_in(if_block3);
				current = true;
			},
			o: function outro(local) {
				transition_out(if_block3);
				current = false;
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div1);
					detach_dev(t6);
					detach_dev(t7);
					detach_dev(if_block4_anchor);
				}

				if (if_block0) if_block0.d();
				if (if_block1) if_block1.d();

				if (if_block2) {
					if_block2.d();
				}

				if (if_block3) if_block3.d(detaching);
				if (if_block4) if_block4.d(detaching);
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

	function formatDate(dateString) {
		return new Date(dateString).toLocaleDateString('en-US', {
			year: 'numeric',
			month: 'long',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	function instance$2($$self, $$props, $$invalidate) {
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('UserProfile', slots, []);
		let profile = null;
		let isLoading = false;
		let error = '';
		let success = '';
		let activeTab = 'profile';

		// Profile editing
		let editingProfile = false;

		let editEmail = '';

		// Settings editing
		let editSettings = {
			two_factor_enabled: false,
			email_notifications: true,
			privacy_mode: false
		};

		// 2FA management
		let show2FASetup = false;

		let show2FADisable = false;

		// Data export
		let exportLoading = false;

		// Account deletion
		let showDeleteConfirm = false;

		let deleteConfirmEmail = '';
		let deleteReason = '';
		let deleteLoading = false;

		onMount(async () => {
			await loadProfile();
		});

		async function loadProfile() {
			$$invalidate(1, isLoading = true);
			$$invalidate(2, error = '');

			try {
				const result = await api.get('/users/profile');

				if (result.success) {
					$$invalidate(0, profile = result.data);
					$$invalidate(6, editEmail = profile?.email || '');

					$$invalidate(7, editSettings = {
						two_factor_enabled: profile?.settings?.two_factor_enabled || false,
						email_notifications: profile?.settings?.email_notifications || true,
						privacy_mode: profile?.settings?.privacy_mode || false
					});
				} else {
					$$invalidate(2, error = result.message || 'Failed to load profile');
				}
			} catch(err) {
				$$invalidate(2, error = err.message || 'Failed to load profile');
			} finally {
				$$invalidate(1, isLoading = false);
			}
		}

		async function updateProfile() {
			if (!profile) return;
			$$invalidate(1, isLoading = true);
			$$invalidate(2, error = '');
			$$invalidate(3, success = '');

			try {
				const result = await api.put('/users/profile', {
					email: editEmail !== profile.email ? editEmail : undefined
				});

				if (result.success) {
					$$invalidate(0, profile = result.data);
					$$invalidate(5, editingProfile = false);
					$$invalidate(3, success = 'Profile updated successfully');

					// Update auth store
					await authActions.fetchProfile();
				} else {
					$$invalidate(2, error = result.message || 'Failed to update profile');
				}
			} catch(err) {
				$$invalidate(2, error = err.message || 'Failed to update profile');
			} finally {
				$$invalidate(1, isLoading = false);
			}
		}

		async function updateSettings() {
			if (!profile) return;
			$$invalidate(1, isLoading = true);
			$$invalidate(2, error = '');
			$$invalidate(3, success = '');

			try {
				const result = await api.put('/users/profile', { settings: editSettings });

				if (result.success) {
					$$invalidate(0, profile = result.data);
					$$invalidate(3, success = 'Settings updated successfully');
				} else {
					$$invalidate(2, error = result.message || 'Failed to update settings');
				}
			} catch(err) {
				$$invalidate(2, error = err.message || 'Failed to update settings');
			} finally {
				$$invalidate(1, isLoading = false);
			}
		}

		async function exportData() {
			$$invalidate(10, exportLoading = true);
			$$invalidate(2, error = '');

			try {
				const result = await api.get('/users/export');

				if (result.success) {
					// Create and download file
					const blob = new Blob([JSON.stringify(result.data, null, 2)], { type: 'application/json' });

					const url = URL.createObjectURL(blob);
					const a = document.createElement('a');
					a.href = url;
					a.download = `no-drake-data-export-${new Date().toISOString().split('T')[0]}.json`;
					document.body.appendChild(a);
					a.click();
					document.body.removeChild(a);
					URL.revokeObjectURL(url);
					$$invalidate(3, success = 'Data exported successfully');
				} else {
					$$invalidate(2, error = result.message || 'Failed to export data');
				}
			} catch(err) {
				$$invalidate(2, error = err.message || 'Failed to export data');
			} finally {
				$$invalidate(10, exportLoading = false);
			}
		}

		async function deleteAccount() {
			if (!profile || deleteConfirmEmail !== profile.email) {
				$$invalidate(2, error = 'Email confirmation does not match');
				return;
			}

			$$invalidate(14, deleteLoading = true);
			$$invalidate(2, error = '');

			try {
				const result = await api.delete('/users/account', {
					confirmation_email: deleteConfirmEmail,
					reason: deleteReason || undefined
				});

				if (result.success) {
					// Account deleted, logout user
					await authActions.logout();
				} else {
					$$invalidate(2, error = result.message || 'Failed to delete account');
				}
			} catch(err) {
				$$invalidate(2, error = err.message || 'Failed to delete account');
			} finally {
				$$invalidate(14, deleteLoading = false);
			}
		}

		function handle2FASetupComplete() {
			$$invalidate(8, show2FASetup = false);
			loadProfile(); // Reload to get updated 2FA status
			$$invalidate(3, success = '2FA enabled successfully');
		}

		function handle2FASetupCancel() {
			$$invalidate(8, show2FASetup = false);
		}

		async function handle2FADisable(code) {
			try {
				const result = await authActions.disable2FA(code);

				if (result.success) {
					$$invalidate(9, show2FADisable = false);
					await loadProfile();
					$$invalidate(3, success = '2FA disabled successfully');
				} else {
					$$invalidate(2, error = result.message || 'Failed to disable 2FA');
				}
			} catch(err) {
				$$invalidate(2, error = err.message || 'Failed to disable 2FA');
			}
		}

		function handleTotpInput(event) {
			const target = event.target;
			const code = target.value;

			if (code.length === 6 && (/^\d{6}$/).test(code)) {
				handle2FADisable(code);
			}
		}

		const writable_props = [];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<UserProfile> was created with unknown prop '${key}'`);
		});

		const click_handler = () => $$invalidate(4, activeTab = 'profile');
		const click_handler_1 = () => $$invalidate(4, activeTab = 'security');
		const click_handler_2 = () => $$invalidate(4, activeTab = 'preferences');
		const click_handler_3 = () => $$invalidate(4, activeTab = 'data');

		function input_input_handler() {
			editEmail = this.value;
			$$invalidate(6, editEmail);
		}

		const click_handler_4 = () => $$invalidate(5, editingProfile = true);

		const click_handler_5 = () => {
			$$invalidate(5, editingProfile = false);
			$$invalidate(6, editEmail = profile?.email || '');
		};

		const click_handler_6 = () => $$invalidate(9, show2FADisable = true);
		const click_handler_7 = () => $$invalidate(8, show2FASetup = true);

		function input0_change_handler() {
			editSettings.email_notifications = this.checked;
			$$invalidate(7, editSettings);
		}

		function input1_change_handler() {
			editSettings.privacy_mode = this.checked;
			$$invalidate(7, editSettings);
		}

		const click_handler_8 = () => $$invalidate(11, showDeleteConfirm = true);

		function input_input_handler_1() {
			deleteConfirmEmail = this.value;
			$$invalidate(12, deleteConfirmEmail);
		}

		function textarea_input_handler() {
			deleteReason = this.value;
			$$invalidate(13, deleteReason);
		}

		const click_handler_9 = () => {
			$$invalidate(11, showDeleteConfirm = false);
			$$invalidate(12, deleteConfirmEmail = '');
			$$invalidate(13, deleteReason = '');
		};

		const click_handler_10 = () => $$invalidate(9, show2FADisable = false);

		$$self.$capture_state = () => ({
			onMount,
			authActions,
			api,
			TwoFactorSetup,
			profile,
			isLoading,
			error,
			success,
			activeTab,
			editingProfile,
			editEmail,
			editSettings,
			show2FASetup,
			show2FADisable,
			exportLoading,
			showDeleteConfirm,
			deleteConfirmEmail,
			deleteReason,
			deleteLoading,
			loadProfile,
			updateProfile,
			updateSettings,
			exportData,
			deleteAccount,
			handle2FASetupComplete,
			handle2FASetupCancel,
			handle2FADisable,
			formatDate,
			handleTotpInput
		});

		$$self.$inject_state = $$props => {
			if ('profile' in $$props) $$invalidate(0, profile = $$props.profile);
			if ('isLoading' in $$props) $$invalidate(1, isLoading = $$props.isLoading);
			if ('error' in $$props) $$invalidate(2, error = $$props.error);
			if ('success' in $$props) $$invalidate(3, success = $$props.success);
			if ('activeTab' in $$props) $$invalidate(4, activeTab = $$props.activeTab);
			if ('editingProfile' in $$props) $$invalidate(5, editingProfile = $$props.editingProfile);
			if ('editEmail' in $$props) $$invalidate(6, editEmail = $$props.editEmail);
			if ('editSettings' in $$props) $$invalidate(7, editSettings = $$props.editSettings);
			if ('show2FASetup' in $$props) $$invalidate(8, show2FASetup = $$props.show2FASetup);
			if ('show2FADisable' in $$props) $$invalidate(9, show2FADisable = $$props.show2FADisable);
			if ('exportLoading' in $$props) $$invalidate(10, exportLoading = $$props.exportLoading);
			if ('showDeleteConfirm' in $$props) $$invalidate(11, showDeleteConfirm = $$props.showDeleteConfirm);
			if ('deleteConfirmEmail' in $$props) $$invalidate(12, deleteConfirmEmail = $$props.deleteConfirmEmail);
			if ('deleteReason' in $$props) $$invalidate(13, deleteReason = $$props.deleteReason);
			if ('deleteLoading' in $$props) $$invalidate(14, deleteLoading = $$props.deleteLoading);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		return [
			profile,
			isLoading,
			error,
			success,
			activeTab,
			editingProfile,
			editEmail,
			editSettings,
			show2FASetup,
			show2FADisable,
			exportLoading,
			showDeleteConfirm,
			deleteConfirmEmail,
			deleteReason,
			deleteLoading,
			updateProfile,
			updateSettings,
			exportData,
			deleteAccount,
			handle2FASetupComplete,
			handle2FASetupCancel,
			handleTotpInput,
			click_handler,
			click_handler_1,
			click_handler_2,
			click_handler_3,
			input_input_handler,
			click_handler_4,
			click_handler_5,
			click_handler_6,
			click_handler_7,
			input0_change_handler,
			input1_change_handler,
			click_handler_8,
			input_input_handler_1,
			textarea_input_handler,
			click_handler_9,
			click_handler_10
		];
	}

	class UserProfile extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance$2, create_fragment$2, safe_not_equal, {}, null, [-1, -1]);

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "UserProfile",
				options,
				id: create_fragment$2.name
			});
		}
	}

	/* src/lib/components/Dashboard.svelte generated by Svelte v4.2.20 */
	const file$1 = "src/lib/components/Dashboard.svelte";

	// (187:42) 
	function create_if_block_7(ctx) {
		let userprofile;
		let current;
		userprofile = new UserProfile({ $$inline: true });

		const block = {
			c: function create() {
				create_component(userprofile.$$.fragment);
			},
			m: function mount(target, anchor) {
				mount_component(userprofile, target, anchor);
				current = true;
			},
			p: noop,
			i: function intro(local) {
				if (current) return;
				transition_in(userprofile.$$.fragment, local);
				current = true;
			},
			o: function outro(local) {
				transition_out(userprofile.$$.fragment, local);
				current = false;
			},
			d: function destroy(detaching) {
				destroy_component(userprofile, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_7.name,
			type: "if",
			source: "(187:42) ",
			ctx
		});

		return block;
	}

	// (185:44) 
	function create_if_block_6(ctx) {
		let communitylists;
		let current;
		communitylists = new CommunityLists({ $$inline: true });

		const block = {
			c: function create() {
				create_component(communitylists.$$.fragment);
			},
			m: function mount(target, anchor) {
				mount_component(communitylists, target, anchor);
				current = true;
			},
			p: noop,
			i: function intro(local) {
				if (current) return;
				transition_in(communitylists.$$.fragment, local);
				current = true;
			},
			o: function outro(local) {
				transition_out(communitylists.$$.fragment, local);
				current = false;
			},
			d: function destroy(detaching) {
				destroy_component(communitylists, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_6.name,
			type: "if",
			source: "(185:44) ",
			ctx
		});

		return block;
	}

	// (183:46) 
	function create_if_block_5(ctx) {
		let enforcementplanning;
		let current;
		enforcementplanning = new EnforcementPlanning({ $$inline: true });

		const block = {
			c: function create() {
				create_component(enforcementplanning.$$.fragment);
			},
			m: function mount(target, anchor) {
				mount_component(enforcementplanning, target, anchor);
				current = true;
			},
			p: noop,
			i: function intro(local) {
				if (current) return;
				transition_in(enforcementplanning.$$.fragment, local);
				current = true;
			},
			o: function outro(local) {
				transition_out(enforcementplanning.$$.fragment, local);
				current = false;
			},
			d: function destroy(detaching) {
				destroy_component(enforcementplanning, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_5.name,
			type: "if",
			source: "(183:46) ",
			ctx
		});

		return block;
	}

	// (181:38) 
	function create_if_block_4(ctx) {
		let dnpmanager;
		let current;
		dnpmanager = new DnpManager({ $$inline: true });

		const block = {
			c: function create() {
				create_component(dnpmanager.$$.fragment);
			},
			m: function mount(target, anchor) {
				mount_component(dnpmanager, target, anchor);
				current = true;
			},
			p: noop,
			i: function intro(local) {
				if (current) return;
				transition_in(dnpmanager.$$.fragment, local);
				current = true;
			},
			o: function outro(local) {
				transition_out(dnpmanager.$$.fragment, local);
				current = false;
			},
			d: function destroy(detaching) {
				destroy_component(dnpmanager, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_4.name,
			type: "if",
			source: "(181:38) ",
			ctx
		});

		return block;
	}

	// (179:46) 
	function create_if_block_3(ctx) {
		let serviceconnections;
		let current;
		serviceconnections = new ServiceConnections({ $$inline: true });

		const block = {
			c: function create() {
				create_component(serviceconnections.$$.fragment);
			},
			m: function mount(target, anchor) {
				mount_component(serviceconnections, target, anchor);
				current = true;
			},
			p: noop,
			i: function intro(local) {
				if (current) return;
				transition_in(serviceconnections.$$.fragment, local);
				current = true;
			},
			o: function outro(local) {
				transition_out(serviceconnections.$$.fragment, local);
				current = false;
			},
			d: function destroy(detaching) {
				destroy_component(serviceconnections, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_3.name,
			type: "if",
			source: "(179:46) ",
			ctx
		});

		return block;
	}

	// (25:4) {#if $currentRoute === 'overview'}
	function create_if_block$1(ctx) {
		let div24;
		let div21;
		let div6;
		let div3;
		let div2;
		let div0;
		let svg0;
		let path0;
		let t0;
		let div1;
		let dl0;
		let dt0;
		let dd0;
		let t2_value = /*$connectedServices*/ ctx[1].length + "";
		let t2;
		let t3;
		let div5;
		let div4;
		let button0;
		let t5;
		let div13;
		let div10;
		let div9;
		let div7;
		let svg1;
		let path1;
		let t6;
		let div8;
		let dl1;
		let dt1;
		let dd1;
		let t8;
		let t9;
		let div12;
		let div11;
		let button1;
		let t11;
		let div20;
		let div17;
		let div16;
		let div14;
		let svg2;
		let path2;
		let svg2_class_value;
		let t12;
		let div15;
		let dl2;
		let dt2;
		let dd2;

		let t14_value = (/*$hasActiveSpotifyConnection*/ ctx[3]
		? 'Active'
		: 'Setup Required') + "";

		let t14;
		let t15;
		let div19;
		let div18;
		let t16;
		let div23;
		let h3;
		let t18;
		let div22;
		let button2;
		let svg3;
		let path3;
		let t19;
		let span;
		let t21;
		let mounted;
		let dispose;

		function select_block_type_1(ctx, dirty) {
			if (!/*$hasActiveSpotifyConnection*/ ctx[3]) return create_if_block_2;
			return create_else_block_1;
		}

		let current_block_type = select_block_type_1(ctx);
		let if_block0 = current_block_type(ctx);

		function select_block_type_2(ctx, dirty) {
			if (/*$hasActiveSpotifyConnection*/ ctx[3] && /*$dnpCount*/ ctx[2] > 0) return create_if_block_1$1;
			return create_else_block$1;
		}

		let current_block_type_1 = select_block_type_2(ctx);
		let if_block1 = current_block_type_1(ctx);

		const block = {
			c: function create() {
				div24 = element("div");
				div21 = element("div");
				div6 = element("div");
				div3 = element("div");
				div2 = element("div");
				div0 = element("div");
				svg0 = svg_element("svg");
				path0 = svg_element("path");
				t0 = space();
				div1 = element("div");
				dl0 = element("dl");
				dt0 = element("dt");
				dt0.textContent = "Connected Services\n                    ";
				dd0 = element("dd");
				t2 = text(t2_value);
				t3 = space();
				div5 = element("div");
				div4 = element("div");
				button0 = element("button");
				button0.textContent = "Manage connections";
				t5 = space();
				div13 = element("div");
				div10 = element("div");
				div9 = element("div");
				div7 = element("div");
				svg1 = svg_element("svg");
				path1 = svg_element("path");
				t6 = space();
				div8 = element("div");
				dl1 = element("dl");
				dt1 = element("dt");
				dt1.textContent = "Blocked Artists\n                    ";
				dd1 = element("dd");
				t8 = text(/*$dnpCount*/ ctx[2]);
				t9 = space();
				div12 = element("div");
				div11 = element("div");
				button1 = element("button");
				button1.textContent = "Manage DNP list";
				t11 = space();
				div20 = element("div");
				div17 = element("div");
				div16 = element("div");
				div14 = element("div");
				svg2 = svg_element("svg");
				path2 = svg_element("path");
				t12 = space();
				div15 = element("div");
				dl2 = element("dl");
				dt2 = element("dt");
				dt2.textContent = "System Status\n                    ";
				dd2 = element("dd");
				t14 = text(t14_value);
				t15 = space();
				div19 = element("div");
				div18 = element("div");
				if_block0.c();
				t16 = space();
				div23 = element("div");
				h3 = element("h3");
				h3.textContent = "Quick Actions";
				t18 = space();
				div22 = element("div");
				button2 = element("button");
				svg3 = svg_element("svg");
				path3 = svg_element("path");
				t19 = space();
				span = element("span");
				span.textContent = "Add Artist to DNP List";
				t21 = space();
				if_block1.c();
				attr_dev(path0, "stroke-linecap", "round");
				attr_dev(path0, "stroke-linejoin", "round");
				attr_dev(path0, "stroke-width", "2");
				attr_dev(path0, "d", "M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1");
				add_location(path0, file$1, 37, 20, 1441);
				attr_dev(svg0, "class", "h-6 w-6 text-gray-400");
				attr_dev(svg0, "fill", "none");
				attr_dev(svg0, "viewBox", "0 0 24 24");
				attr_dev(svg0, "stroke", "currentColor");
				add_location(svg0, file$1, 36, 18, 1331);
				attr_dev(div0, "class", "flex-shrink-0");
				add_location(div0, file$1, 35, 16, 1285);
				attr_dev(dt0, "class", "text-sm font-medium text-gray-500 truncate");
				add_location(dt0, file$1, 42, 20, 1786);
				attr_dev(dd0, "class", "text-lg font-medium text-gray-900");
				add_location(dd0, file$1, 45, 20, 1929);
				add_location(dl0, file$1, 41, 18, 1761);
				attr_dev(div1, "class", "ml-5 w-0 flex-1");
				add_location(div1, file$1, 40, 16, 1713);
				attr_dev(div2, "class", "flex items-center");
				add_location(div2, file$1, 34, 14, 1237);
				attr_dev(div3, "class", "p-5");
				add_location(div3, file$1, 33, 12, 1205);
				attr_dev(button0, "class", "font-medium text-indigo-700 hover:text-indigo-900");
				add_location(button0, file$1, 54, 16, 2238);
				attr_dev(div4, "class", "text-sm");
				add_location(div4, file$1, 53, 14, 2200);
				attr_dev(div5, "class", "bg-gray-50 px-5 py-3");
				add_location(div5, file$1, 52, 12, 2151);
				attr_dev(div6, "class", "bg-white overflow-hidden shadow rounded-lg");
				add_location(div6, file$1, 32, 10, 1136);
				attr_dev(path1, "stroke-linecap", "round");
				attr_dev(path1, "stroke-linejoin", "round");
				attr_dev(path1, "stroke-width", "2");
				attr_dev(path1, "d", "M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728L5.636 5.636m12.728 12.728L5.636 5.636");
				add_location(path1, file$1, 70, 20, 2872);
				attr_dev(svg1, "class", "h-6 w-6 text-gray-400");
				attr_dev(svg1, "fill", "none");
				attr_dev(svg1, "viewBox", "0 0 24 24");
				attr_dev(svg1, "stroke", "currentColor");
				add_location(svg1, file$1, 69, 18, 2762);
				attr_dev(div7, "class", "flex-shrink-0");
				add_location(div7, file$1, 68, 16, 2716);
				attr_dev(dt1, "class", "text-sm font-medium text-gray-500 truncate");
				add_location(dt1, file$1, 75, 20, 3173);
				attr_dev(dd1, "class", "text-lg font-medium text-gray-900");
				add_location(dd1, file$1, 78, 20, 3313);
				add_location(dl1, file$1, 74, 18, 3148);
				attr_dev(div8, "class", "ml-5 w-0 flex-1");
				add_location(div8, file$1, 73, 16, 3100);
				attr_dev(div9, "class", "flex items-center");
				add_location(div9, file$1, 67, 14, 2668);
				attr_dev(div10, "class", "p-5");
				add_location(div10, file$1, 66, 12, 2636);
				attr_dev(button1, "class", "font-medium text-indigo-700 hover:text-indigo-900");
				add_location(button1, file$1, 87, 16, 3606);
				attr_dev(div11, "class", "text-sm");
				add_location(div11, file$1, 86, 14, 3568);
				attr_dev(div12, "class", "bg-gray-50 px-5 py-3");
				add_location(div12, file$1, 85, 12, 3519);
				attr_dev(div13, "class", "bg-white overflow-hidden shadow rounded-lg");
				add_location(div13, file$1, 65, 10, 2567);
				attr_dev(path2, "stroke-linecap", "round");
				attr_dev(path2, "stroke-linejoin", "round");
				attr_dev(path2, "stroke-width", "2");
				attr_dev(path2, "d", "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z");
				add_location(path2, file$1, 103, 20, 4280);

				attr_dev(svg2, "class", svg2_class_value = "h-6 w-6 " + (/*$hasActiveSpotifyConnection*/ ctx[3]
				? 'text-green-400'
				: 'text-gray-400'));

				attr_dev(svg2, "fill", "none");
				attr_dev(svg2, "viewBox", "0 0 24 24");
				attr_dev(svg2, "stroke", "currentColor");
				add_location(svg2, file$1, 102, 18, 4117);
				attr_dev(div14, "class", "flex-shrink-0");
				add_location(div14, file$1, 101, 16, 4071);
				attr_dev(dt2, "class", "text-sm font-medium text-gray-500 truncate");
				add_location(dt2, file$1, 108, 20, 4540);
				attr_dev(dd2, "class", "text-lg font-medium text-gray-900");
				add_location(dd2, file$1, 111, 20, 4678);
				add_location(dl2, file$1, 107, 18, 4515);
				attr_dev(div15, "class", "ml-5 w-0 flex-1");
				add_location(div15, file$1, 106, 16, 4467);
				attr_dev(div16, "class", "flex items-center");
				add_location(div16, file$1, 100, 14, 4023);
				attr_dev(div17, "class", "p-5");
				add_location(div17, file$1, 99, 12, 3991);
				attr_dev(div18, "class", "text-sm");
				add_location(div18, file$1, 119, 14, 4981);
				attr_dev(div19, "class", "bg-gray-50 px-5 py-3");
				add_location(div19, file$1, 118, 12, 4932);
				attr_dev(div20, "class", "bg-white overflow-hidden shadow rounded-lg");
				add_location(div20, file$1, 98, 10, 3922);
				attr_dev(div21, "class", "grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3");
				add_location(div21, file$1, 30, 8, 1016);
				attr_dev(h3, "class", "text-lg leading-6 font-medium text-gray-900 mb-4");
				add_location(h3, file$1, 137, 10, 5561);
				attr_dev(path3, "stroke-linecap", "round");
				attr_dev(path3, "stroke-linejoin", "round");
				attr_dev(path3, "stroke-width", "2");
				attr_dev(path3, "d", "M12 6v6m0 0v6m0-6h6m-6 0H6");
				add_location(path3, file$1, 146, 16, 6146);
				attr_dev(svg3, "class", "mx-auto h-8 w-8 text-gray-400");
				attr_dev(svg3, "fill", "none");
				attr_dev(svg3, "viewBox", "0 0 24 24");
				attr_dev(svg3, "stroke", "currentColor");
				add_location(svg3, file$1, 145, 14, 6032);
				attr_dev(span, "class", "mt-2 block text-sm font-medium text-gray-900");
				add_location(span, file$1, 148, 14, 6285);
				attr_dev(button2, "class", "relative block w-full border-2 border-gray-300 border-dashed rounded-lg p-6 text-center hover:border-gray-400 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500");
				add_location(button2, file$1, 141, 12, 5739);
				attr_dev(div22, "class", "grid grid-cols-1 gap-4 sm:grid-cols-2");
				add_location(div22, file$1, 140, 10, 5675);
				attr_dev(div23, "class", "mt-8");
				add_location(div23, file$1, 136, 8, 5532);
				attr_dev(div24, "class", "px-4 py-6 sm:px-0");
				add_location(div24, file$1, 29, 6, 976);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div24, anchor);
				append_dev(div24, div21);
				append_dev(div21, div6);
				append_dev(div6, div3);
				append_dev(div3, div2);
				append_dev(div2, div0);
				append_dev(div0, svg0);
				append_dev(svg0, path0);
				append_dev(div2, t0);
				append_dev(div2, div1);
				append_dev(div1, dl0);
				append_dev(dl0, dt0);
				append_dev(dl0, dd0);
				append_dev(dd0, t2);
				append_dev(div6, t3);
				append_dev(div6, div5);
				append_dev(div5, div4);
				append_dev(div4, button0);
				append_dev(div21, t5);
				append_dev(div21, div13);
				append_dev(div13, div10);
				append_dev(div10, div9);
				append_dev(div9, div7);
				append_dev(div7, svg1);
				append_dev(svg1, path1);
				append_dev(div9, t6);
				append_dev(div9, div8);
				append_dev(div8, dl1);
				append_dev(dl1, dt1);
				append_dev(dl1, dd1);
				append_dev(dd1, t8);
				append_dev(div13, t9);
				append_dev(div13, div12);
				append_dev(div12, div11);
				append_dev(div11, button1);
				append_dev(div21, t11);
				append_dev(div21, div20);
				append_dev(div20, div17);
				append_dev(div17, div16);
				append_dev(div16, div14);
				append_dev(div14, svg2);
				append_dev(svg2, path2);
				append_dev(div16, t12);
				append_dev(div16, div15);
				append_dev(div15, dl2);
				append_dev(dl2, dt2);
				append_dev(dl2, dd2);
				append_dev(dd2, t14);
				append_dev(div20, t15);
				append_dev(div20, div19);
				append_dev(div19, div18);
				if_block0.m(div18, null);
				append_dev(div24, t16);
				append_dev(div24, div23);
				append_dev(div23, h3);
				append_dev(div23, t18);
				append_dev(div23, div22);
				append_dev(div22, button2);
				append_dev(button2, svg3);
				append_dev(svg3, path3);
				append_dev(button2, t19);
				append_dev(button2, span);
				append_dev(div22, t21);
				if_block1.m(div22, null);

				if (!mounted) {
					dispose = [
						listen_dev(button0, "click", /*click_handler*/ ctx[5], false, false, false, false),
						listen_dev(button1, "click", /*click_handler_1*/ ctx[6], false, false, false, false),
						listen_dev(button2, "click", /*click_handler_3*/ ctx[8], false, false, false, false)
					];

					mounted = true;
				}
			},
			p: function update(ctx, dirty) {
				if (dirty & /*$connectedServices*/ 2 && t2_value !== (t2_value = /*$connectedServices*/ ctx[1].length + "")) set_data_dev(t2, t2_value);
				if (dirty & /*$dnpCount*/ 4) set_data_dev(t8, /*$dnpCount*/ ctx[2]);

				if (dirty & /*$hasActiveSpotifyConnection*/ 8 && svg2_class_value !== (svg2_class_value = "h-6 w-6 " + (/*$hasActiveSpotifyConnection*/ ctx[3]
				? 'text-green-400'
				: 'text-gray-400'))) {
					attr_dev(svg2, "class", svg2_class_value);
				}

				if (dirty & /*$hasActiveSpotifyConnection*/ 8 && t14_value !== (t14_value = (/*$hasActiveSpotifyConnection*/ ctx[3]
				? 'Active'
				: 'Setup Required') + "")) set_data_dev(t14, t14_value);

				if (current_block_type === (current_block_type = select_block_type_1(ctx)) && if_block0) {
					if_block0.p(ctx, dirty);
				} else {
					if_block0.d(1);
					if_block0 = current_block_type(ctx);

					if (if_block0) {
						if_block0.c();
						if_block0.m(div18, null);
					}
				}

				if (current_block_type_1 === (current_block_type_1 = select_block_type_2(ctx)) && if_block1) {
					if_block1.p(ctx, dirty);
				} else {
					if_block1.d(1);
					if_block1 = current_block_type_1(ctx);

					if (if_block1) {
						if_block1.c();
						if_block1.m(div22, null);
					}
				}
			},
			i: noop,
			o: noop,
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div24);
				}

				if_block0.d();
				if_block1.d();
				mounted = false;
				run_all(dispose);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block$1.name,
			type: "if",
			source: "(25:4) {#if $currentRoute === 'overview'}",
			ctx
		});

		return block;
	}

	// (125:16) {:else}
	function create_else_block_1(ctx) {
		let span;

		const block = {
			c: function create() {
				span = element("span");
				span.textContent = "Ready to use";
				attr_dev(span, "class", "text-green-700");
				add_location(span, file$1, 128, 18, 5349);
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
			id: create_else_block_1.name,
			type: "else",
			source: "(125:16) {:else}",
			ctx
		});

		return block;
	}

	// (118:16) {#if !$hasActiveSpotifyConnection}
	function create_if_block_2(ctx) {
		let button;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				button = element("button");
				button.textContent = "Connect Spotify";
				attr_dev(button, "class", "font-medium text-indigo-700 hover:text-indigo-900");
				add_location(button, file$1, 121, 18, 5072);
			},
			m: function mount(target, anchor) {
				insert_dev(target, button, anchor);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*click_handler_2*/ ctx[7], false, false, false, false);
					mounted = true;
				}
			},
			p: noop,
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
			id: create_if_block_2.name,
			type: "if",
			source: "(118:16) {#if !$hasActiveSpotifyConnection}",
			ctx
		});

		return block;
	}

	// (163:12) {:else}
	function create_else_block$1(ctx) {
		let button;
		let svg;
		let path;
		let t0;
		let span;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				button = element("button");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				span = element("span");
				span.textContent = "Connect Spotify";
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1");
				add_location(path, file$1, 171, 18, 7748);
				attr_dev(svg, "class", "mx-auto h-8 w-8 text-gray-400");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$1, 170, 16, 7632);
				attr_dev(span, "class", "mt-2 block text-sm font-medium text-gray-900");
				add_location(span, file$1, 173, 16, 7995);
				attr_dev(button, "class", "relative block w-full border-2 border-gray-300 border-dashed rounded-lg p-6 text-center hover:border-gray-400 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500");
				add_location(button, file$1, 166, 14, 7323);
			},
			m: function mount(target, anchor) {
				insert_dev(target, button, anchor);
				append_dev(button, svg);
				append_dev(svg, path);
				append_dev(button, t0);
				append_dev(button, span);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*click_handler_5*/ ctx[10], false, false, false, false);
					mounted = true;
				}
			},
			p: noop,
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
			id: create_else_block$1.name,
			type: "else",
			source: "(163:12) {:else}",
			ctx
		});

		return block;
	}

	// (151:12) {#if $hasActiveSpotifyConnection && $dnpCount > 0}
	function create_if_block_1$1(ctx) {
		let button;
		let svg;
		let path;
		let t0;
		let span;
		let mounted;
		let dispose;

		const block = {
			c: function create() {
				button = element("button");
				svg = svg_element("svg");
				path = svg_element("path");
				t0 = space();
				span = element("span");
				span.textContent = "Plan Enforcement";
				attr_dev(path, "stroke-linecap", "round");
				attr_dev(path, "stroke-linejoin", "round");
				attr_dev(path, "stroke-width", "2");
				attr_dev(path, "d", "M14.828 14.828a4 4 0 01-5.656 0M9 10h1.01M15 10h1.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z");
				add_location(path, file$1, 159, 18, 6943);
				attr_dev(svg, "class", "mx-auto h-8 w-8 text-gray-400");
				attr_dev(svg, "fill", "none");
				attr_dev(svg, "viewBox", "0 0 24 24");
				attr_dev(svg, "stroke", "currentColor");
				add_location(svg, file$1, 158, 16, 6827);
				attr_dev(span, "class", "mt-2 block text-sm font-medium text-gray-900");
				add_location(span, file$1, 161, 16, 7146);
				attr_dev(button, "class", "relative block w-full border-2 border-gray-300 border-dashed rounded-lg p-6 text-center hover:border-gray-400 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500");
				add_location(button, file$1, 154, 14, 6518);
			},
			m: function mount(target, anchor) {
				insert_dev(target, button, anchor);
				append_dev(button, svg);
				append_dev(svg, path);
				append_dev(button, t0);
				append_dev(button, span);

				if (!mounted) {
					dispose = listen_dev(button, "click", /*click_handler_4*/ ctx[9], false, false, false, false);
					mounted = true;
				}
			},
			p: noop,
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
			id: create_if_block_1$1.name,
			type: "if",
			source: "(151:12) {#if $hasActiveSpotifyConnection && $dnpCount > 0}",
			ctx
		});

		return block;
	}

	function create_fragment$1(ctx) {
		let div;
		let navigation;
		let t;
		let main;
		let current_block_type_index;
		let if_block;
		let current;
		navigation = new Navigation({ $$inline: true });

		const if_block_creators = [
			create_if_block$1,
			create_if_block_3,
			create_if_block_4,
			create_if_block_5,
			create_if_block_6,
			create_if_block_7
		];

		const if_blocks = [];

		function select_block_type(ctx, dirty) {
			if (/*$currentRoute*/ ctx[0] === 'overview') return 0;
			if (/*$currentRoute*/ ctx[0] === 'connections') return 1;
			if (/*$currentRoute*/ ctx[0] === 'dnp') return 2;
			if (/*$currentRoute*/ ctx[0] === 'enforcement') return 3;
			if (/*$currentRoute*/ ctx[0] === 'community') return 4;
			if (/*$currentRoute*/ ctx[0] === 'profile') return 5;
			return -1;
		}

		if (~(current_block_type_index = select_block_type(ctx))) {
			if_block = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);
		}

		const block = {
			c: function create() {
				div = element("div");
				create_component(navigation.$$.fragment);
				t = space();
				main = element("main");
				if (if_block) if_block.c();
				attr_dev(main, "class", "max-w-7xl mx-auto py-6 sm:px-6 lg:px-8");
				add_location(main, file$1, 26, 2, 849);
				attr_dev(div, "class", "min-h-screen bg-gray-50");
				add_location(div, file$1, 22, 0, 767);
			},
			l: function claim(nodes) {
				throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
			},
			m: function mount(target, anchor) {
				insert_dev(target, div, anchor);
				mount_component(navigation, div, null);
				append_dev(div, t);
				append_dev(div, main);

				if (~current_block_type_index) {
					if_blocks[current_block_type_index].m(main, null);
				}

				current = true;
			},
			p: function update(ctx, [dirty]) {
				let previous_block_index = current_block_type_index;
				current_block_type_index = select_block_type(ctx);

				if (current_block_type_index === previous_block_index) {
					if (~current_block_type_index) {
						if_blocks[current_block_type_index].p(ctx, dirty);
					}
				} else {
					if (if_block) {
						group_outros();

						transition_out(if_blocks[previous_block_index], 1, 1, () => {
							if_blocks[previous_block_index] = null;
						});

						check_outros();
					}

					if (~current_block_type_index) {
						if_block = if_blocks[current_block_type_index];

						if (!if_block) {
							if_block = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);
							if_block.c();
						} else {
							if_block.p(ctx, dirty);
						}

						transition_in(if_block, 1);
						if_block.m(main, null);
					} else {
						if_block = null;
					}
				}
			},
			i: function intro(local) {
				if (current) return;
				transition_in(navigation.$$.fragment, local);
				transition_in(if_block);
				current = true;
			},
			o: function outro(local) {
				transition_out(navigation.$$.fragment, local);
				transition_out(if_block);
				current = false;
			},
			d: function destroy(detaching) {
				if (detaching) {
					detach_dev(div);
				}

				destroy_component(navigation);

				if (~current_block_type_index) {
					if_blocks[current_block_type_index].d();
				}
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

	function instance$1($$self, $$props, $$invalidate) {
		let $currentRoute;
		let $connectedServices;
		let $dnpCount;
		let $hasActiveSpotifyConnection;
		validate_store(currentRoute, 'currentRoute');
		component_subscribe($$self, currentRoute, $$value => $$invalidate(0, $currentRoute = $$value));
		validate_store(connectedServices, 'connectedServices');
		component_subscribe($$self, connectedServices, $$value => $$invalidate(1, $connectedServices = $$value));
		validate_store(dnpCount, 'dnpCount');
		component_subscribe($$self, dnpCount, $$value => $$invalidate(2, $dnpCount = $$value));
		validate_store(hasActiveSpotifyConnection, 'hasActiveSpotifyConnection');
		component_subscribe($$self, hasActiveSpotifyConnection, $$value => $$invalidate(3, $hasActiveSpotifyConnection = $$value));
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('Dashboard', slots, []);

		onMount(async () => {
			await connectionActions.fetchConnections();
			await dnpActions.fetchDnpList();
		});

		function setActiveTab(tab) {
			router.navigate(tab);
		}

		const writable_props = [];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console.warn(`<Dashboard> was created with unknown prop '${key}'`);
		});

		const click_handler = () => setActiveTab('connections');
		const click_handler_1 = () => setActiveTab('dnp');
		const click_handler_2 = () => setActiveTab('connections');
		const click_handler_3 = () => setActiveTab('dnp');
		const click_handler_4 = () => setActiveTab('enforcement');
		const click_handler_5 = () => setActiveTab('connections');

		$$self.$capture_state = () => ({
			onMount,
			connectionActions,
			connectedServices,
			hasActiveSpotifyConnection,
			dnpActions,
			dnpCount,
			router,
			currentRoute,
			Navigation,
			ServiceConnections,
			DnpManager,
			EnforcementPlanning,
			CommunityLists,
			UserProfile,
			setActiveTab,
			$currentRoute,
			$connectedServices,
			$dnpCount,
			$hasActiveSpotifyConnection
		});

		return [
			$currentRoute,
			$connectedServices,
			$dnpCount,
			$hasActiveSpotifyConnection,
			setActiveTab,
			click_handler,
			click_handler_1,
			click_handler_2,
			click_handler_3,
			click_handler_4,
			click_handler_5
		];
	}

	class Dashboard extends SvelteComponentDev {
		constructor(options) {
			super(options);
			init(this, options, instance$1, create_fragment$1, safe_not_equal, {});

			dispatch_dev("SvelteRegisterComponent", {
				component: this,
				tagName: "Dashboard",
				options,
				id: create_fragment$1.name
			});
		}
	}

	/* src/App.svelte generated by Svelte v4.2.20 */

	const { console: console_1 } = globals;
	const file = "src/App.svelte";

	// (36:0) {:else}
	function create_else_block(ctx) {
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
			id: create_else_block.name,
			type: "else",
			source: "(36:0) {:else}",
			ctx
		});

		return block;
	}

	// (34:27) 
	function create_if_block_1(ctx) {
		let dashboard;
		let current;
		dashboard = new Dashboard({ $$inline: true });

		const block = {
			c: function create() {
				create_component(dashboard.$$.fragment);
			},
			m: function mount(target, anchor) {
				mount_component(dashboard, target, anchor);
				current = true;
			},
			i: function intro(local) {
				if (current) return;
				transition_in(dashboard.$$.fragment, local);
				current = true;
			},
			o: function outro(local) {
				transition_out(dashboard.$$.fragment, local);
				current = false;
			},
			d: function destroy(detaching) {
				destroy_component(dashboard, detaching);
			}
		};

		dispatch_dev("SvelteRegisterBlock", {
			block,
			id: create_if_block_1.name,
			type: "if",
			source: "(34:27) ",
			ctx
		});

		return block;
	}

	// (27:0) {#if !isInitialized}
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
				attr_dev(div0, "class", "animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600 mx-auto");
				add_location(div0, file, 31, 3, 946);
				attr_dev(p, "class", "mt-4 text-gray-600");
				add_location(p, file, 32, 3, 1042);
				attr_dev(div1, "class", "text-center");
				add_location(div1, file, 30, 2, 917);
				attr_dev(div2, "class", "min-h-screen flex items-center justify-center bg-gray-50");
				add_location(div2, file, 29, 1, 844);
			},
			m: function mount(target, anchor) {
				insert_dev(target, div2, anchor);
				append_dev(div2, div1);
				append_dev(div1, div0);
				append_dev(div1, t0);
				append_dev(div1, p);
			},
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
			source: "(27:0) {#if !isInitialized}",
			ctx
		});

		return block;
	}

	function create_fragment(ctx) {
		let current_block_type_index;
		let if_block;
		let if_block_anchor;
		let current;
		const if_block_creators = [create_if_block, create_if_block_1, create_else_block];
		const if_blocks = [];

		function select_block_type(ctx, dirty) {
			if (!/*isInitialized*/ ctx[0]) return 0;
			if (/*$isAuthenticated*/ ctx[1]) return 1;
			return 2;
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
			id: create_fragment.name,
			type: "component",
			source: "",
			ctx
		});

		return block;
	}

	function instance($$self, $$props, $$invalidate) {
		let $isAuthenticated;
		validate_store(isAuthenticated, 'isAuthenticated');
		component_subscribe($$self, isAuthenticated, $$value => $$invalidate(1, $isAuthenticated = $$value));
		let { $$slots: slots = {}, $$scope } = $$props;
		validate_slots('App', slots, []);
		let isInitialized = false;

		onMount(async () => {
			console.log('App mounting...');

			try {
				// Initialize router
				router.init();

				console.log('Router initialized');

				// Initialize auth state
				await authActions.fetchProfile();

				console.log('Auth profile fetched');
			} catch(error) {
				console.error('Error during app initialization:', error);
			} finally {
				$$invalidate(0, isInitialized = true);
				console.log('App initialized, isAuthenticated:', $isAuthenticated);
			}
		});

		const writable_props = [];

		Object.keys($$props).forEach(key => {
			if (!~writable_props.indexOf(key) && key.slice(0, 2) !== '$$' && key !== 'slot') console_1.warn(`<App> was created with unknown prop '${key}'`);
		});

		$$self.$capture_state = () => ({
			onMount,
			isAuthenticated,
			authActions,
			router,
			Login,
			Dashboard,
			isInitialized,
			$isAuthenticated
		});

		$$self.$inject_state = $$props => {
			if ('isInitialized' in $$props) $$invalidate(0, isInitialized = $$props.isInitialized);
		};

		if ($$props && "$$inject" in $$props) {
			$$self.$inject_state($$props.$$inject);
		}

		return [isInitialized, $isAuthenticated];
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
