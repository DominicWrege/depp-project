document.body.onload = (e) => {
	document.querySelector("#solutionEditor").style = editorStyle("28em");
	document.querySelector("#customScriptEditor").style = editorStyle("14em");
	const solutionElement = document.querySelector("#hiddenSolutionText");
	if (solutionElement) {
		// for exsisting assignment
		const ScriptTypeElement = document.querySelector("#hiddenScriptType");
		const CustomScriptElement = document.querySelector(
			"#hiddenCustomScriptText"
		);
		const lang_text = getText(ScriptTypeElement);
		initEditor(
			getText(solutionElement),
			scriptToLanguage(lang_text),
			getText(CustomScriptElement)
		);
	} else {
		// for exsisting new assignment
		initEditor("", "shell", "");
	}

	function initEditor(solutionText, solutionlanguage, customScriptText) {
		require(["vs/editor/editor.main"], () => {
			document.querySelector("#loadingPlaceholder").remove();
			window.soluitionEditor = createEditor(
				"#solutionEditor",
				solutionText,
				solutionlanguage
			);
			window.customScriptEditor = createEditor(
				"#customScriptEditor",
				customScriptText,
				"shell"
			);
			document
				.querySelector("#AssignmentForm")
				.addEventListener("formdata", (e) => {
					e.formData.append(
						"solution",
						window.soluitionEditor.getModel().getValue()
					);
					e.formData.append(
						"custom_script",
						window.customScriptEditor.getModel().getValue()
					);
				});
		});
	}
};

function editorStyle(height) {
	return `width:auto;height:${height};border:1px solid lightgray;margin-bottom: 0.3em;`;
}

function getText(element) {
	return element.textContent.trim();
}

function createEditor(selector, text, language) {
	return monaco.editor.create(document.querySelector(selector), {
		theme: "vs-light",
		model: monaco.editor.createModel(text, language),
		wordWrap: "on",
		automaticLayout: true,
		lineNumbersMinChars: 3,
		scrollBeyondLastLine: false,
		suggestOnTriggerCharacters: true,
		minimap: {
			enabled: true,
		},
		scrollbar: {
			vertical: "auto",
		},
		colorDecorators: true,
		tabCompletion: "on"
	});
}

function languageChanged(select) {
	changeEditorLanguage(scriptToLanguage(select.value));
}

function scriptToLanguage(lang) {
	switch (lang) {
		case "Python3":
			return "python";
			break;
		case "PowerShell":
			return "powershell";
			break;
		case "Batch":
			return "bat";
		default:
			return "shell";
	}
}

function changeEditorLanguage(lang) {
	if (window.soluitionEditor) {
		const model = window.soluitionEditor.getModel();
		monaco.editor.setModelLanguage(model, lang);
	}
}
