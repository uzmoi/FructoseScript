import type {
  CodeMapping,
  LanguagePlugin,
  VirtualCode,
} from "@volar/language-core";
import {} from "@volar/typescript";
import { isFructoseFilePath } from "./utils";

const LANGUAGE_ID = "fructose";

// ひとまず動作させるためのダミー
const transpile = (source: string) => {
  let text = "";

  const mapping: CodeMapping = {
    data: { navigation: true },
    sourceOffsets: [],
    generatedOffsets: [],
    lengths: [],
  };

  for (const match of source.matchAll(/\w+/g)) {
    text += "export const ";

    mapping.sourceOffsets.push(match.index);
    mapping.generatedOffsets.push(text.length);
    mapping.lengths.push(match[0].length);

    text += `${match[0]}: unknown;\n`;
  }

  return { text, mapping };
};

export const createFructoseLanguagePlugin = (
  ts: typeof import("typescript"),
): LanguagePlugin<string, VirtualCode> => {
  return {
    getLanguageId(scriptId) {
      if (isFructoseFilePath(scriptId)) {
        return LANGUAGE_ID;
      }
    },

    createVirtualCode(_scriptId, languageId, snapshot, _ctx) {
      if (languageId !== LANGUAGE_ID) return;

      const length = snapshot.getLength();
      const source = snapshot.getText(0, length);
      const { text, mapping } = transpile(source);

      return {
        id: "main",
        languageId: "typescript",
        snapshot: {
          getText: (start, end) => text.slice(start, end),
          getLength: () => text.length,
          getChangeRange: () => undefined,
        },
        mappings: [mapping],
      };
    },

    typescript: {
      extraFileExtensions: [
        {
          extension: "fru",
          isMixedContent: true,
          scriptKind: ts.ScriptKind.TS,
        },
      ],
      getServiceScript(root) {
        return {
          code: root,
          extension: ts.Extension.Ts,
          scriptKind: ts.ScriptKind.TS,
        };
      },
    },
  };
};
