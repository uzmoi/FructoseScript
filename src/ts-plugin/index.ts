import { createLanguageServicePlugin } from "@volar/typescript/lib/quickstart/createLanguageServicePlugin";
import { createFructoseLanguagePlugin } from "./language-plugin";

const plugin = createLanguageServicePlugin((ts, info) => {
  info.project.projectService.logger.info("[fructose] load");

  return {
    languagePlugins: [createFructoseLanguagePlugin(ts)],
  };
});

export default plugin;
