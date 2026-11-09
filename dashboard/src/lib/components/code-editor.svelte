<script lang="ts">
	import { onMount } from 'svelte';
	import { basicSetup } from 'codemirror';
	import { EditorState, Compartment } from '@codemirror/state';
	import { EditorView } from '@codemirror/view';
	import { markdown } from '@codemirror/lang-markdown';
	import { languages } from '@codemirror/language-data';
	import { oneDark } from '@codemirror/theme-one-dark';
	import { redo as redoCommand, undo as undoCommand } from '@codemirror/commands';
	import { htmlToMarkdown, looksLikeMarkup } from '$lib/html-to-markdown';
	import { mode } from 'mode-watcher';

	let { value = $bindable('') }: { value?: string } = $props();

	let host: HTMLDivElement;
	let view: EditorView | null = null;

	const themeCompartment = new Compartment();

	const baseTheme = EditorView.theme({
		'&': { fontSize: '13px', backgroundColor: 'transparent' },
		'.cm-scroller': {
			fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Consolas, monospace',
			lineHeight: '1.65'
		},
		'.cm-content': { padding: '12px 0' },
		'&.cm-focused': { outline: 'none' },
		'.cm-gutters': { backgroundColor: 'transparent', border: 'none', color: '#a1a1aa' }
	});

	onMount(() => {
		view = new EditorView({
			parent: host,
			state: EditorState.create({
				doc: value,
				extensions: [
					basicSetup,
					markdown({ codeLanguages: languages }),
					baseTheme,
					themeCompartment.of(mode.current === 'dark' ? oneDark : []),
					EditorView.domEventHandlers({
						paste: (event, view) => {
							const data = event.clipboardData;
							if (!data) return false;

							const html = data.getData('text/html');
							if (!html || !looksLikeMarkup(html)) return false;

							event.preventDefault();

							const markdown = htmlToMarkdown(html);
							if (markdown) {
								view.dispatch(view.state.replaceSelection(markdown));
							}

							return true;
						}
					}),
					EditorView.updateListener.of((update) => {
						if (update.docChanged) value = update.state.doc.toString();
					})
				]
			})
		});

		return () => view?.destroy();
	});

	$effect(() => {
		const dark = mode.current === 'dark';
		if (view) {
			view.dispatch({ effects: themeCompartment.reconfigure(dark ? oneDark : []) });
		}
	});

	$effect(() => {
		if (!view) return;
		if (value !== view.state.doc.toString()) {
			view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: value } });
		}
	});

	export function insert(text: string) {
		if (!view) return;
		view.dispatch(view.state.replaceSelection(text));
		view.focus();
	}

	export function undo() {
		if (!view) return;
		undoCommand(view);
		view.focus();
	}

	export function redo() {
		if (!view) return;
		redoCommand(view);
		view.focus();
	}
</script>

<div bind:this={host} class="h-[58vh] overflow-auto rounded-lg border border-input"></div>
