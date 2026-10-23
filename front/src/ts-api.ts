// Recorte estrutural da Compiler API que o ST0 usa. O `typescript` vem do projeto
// analisado (5.1 a 6.0), então não há tipos importáveis em tempo de build.
export interface TsDiagnostic {
  readonly category: number;
  readonly code: number;
  readonly messageText: unknown;
}

export interface TsSourceFile {
  readonly fileName: string;
  readonly isDeclarationFile: boolean;
}

export type TsOptions = Record<string, unknown>;
export type TsWriteFile = (
  fileName: string,
  text: string,
  writeByteOrderMark: boolean,
  onError?: (message: string) => void,
) => void;

export interface TsHost {
  writeFile: TsWriteFile;
}

export interface TsProgram {
  getSourceFiles(): readonly TsSourceFile[];
  isSourceFileFromExternalLibrary(file: TsSourceFile): boolean;
  isSourceFileDefaultLibrary(file: TsSourceFile): boolean;
  emit(file?: TsSourceFile, writeFile?: TsWriteFile): { emitSkipped: boolean };
}

export interface TsParsedConfig {
  readonly options: TsOptions;
  readonly fileNames: readonly string[];
  readonly projectReferences?: readonly unknown[];
  readonly errors: readonly TsDiagnostic[];
}

export interface TsApi {
  readonly version: string;
  readonly sys: { readFile(path: string): string | undefined };
  readonly DiagnosticCategory: { readonly Error: number };
  readConfigFile(
    path: string,
    readFile: (path: string) => string | undefined,
  ): { config?: unknown; error?: TsDiagnostic };
  parseJsonConfigFileContent(
    json: unknown,
    host: unknown,
    basePath: string,
    existingOptions: TsOptions | undefined,
    configFileName: string,
  ): TsParsedConfig;
  createCompilerHost(options: TsOptions): TsHost;
  createProgram(init: {
    rootNames: readonly string[];
    options: TsOptions;
    host: TsHost;
    projectReferences?: readonly unknown[];
    configFileParsingDiagnostics?: readonly TsDiagnostic[];
  }): TsProgram;
  getPreEmitDiagnostics(program: TsProgram): readonly TsDiagnostic[];
  flattenDiagnosticMessageText(messageText: unknown, newLine: string): string;
}
