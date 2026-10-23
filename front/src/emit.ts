import type { ProjectConfig } from './config.ts';
import type { TsApi, TsDiagnostic, TsOptions, TsSourceFile, TsWriteFile } from './ts-api.ts';

export interface EmitResult {
  readonly options: TsOptions;
  readonly sourceFiles: readonly string[];
  readonly writtenFiles: readonly string[];
  readonly diagnostics: readonly TsDiagnostic[];
}

// Só o `outDir` muda: qualquer outra opção do projeto vai intacta para o emit.
export function emitProject(ts: TsApi, config: ProjectConfig, outDir: string): EmitResult {
  const options: TsOptions = { ...config.parsed.options, outDir };
  const host = ts.createCompilerHost(options);
  const program = ts.createProgram({
    rootNames: config.parsed.fileNames,
    options,
    host,
    projectReferences: config.parsed.projectReferences,
    configFileParsingDiagnostics: config.parsed.errors,
  });
  const writtenFiles: string[] = [];
  const writeFile: TsWriteFile = (fileName, text, writeByteOrderMark, onError) => {
    host.writeFile(fileName, text, writeByteOrderMark, onError);
    writtenFiles.push(fileName);
  };
  const sources = emittableSources(program);
  // `outFile` junta tudo num bundle: emitir por arquivo não se aplica.
  if (options.outFile === undefined) {
    for (const file of sources) program.emit(file, writeFile);
  } else {
    program.emit(undefined, writeFile);
  }
  return {
    options,
    sourceFiles: sources.map((file) => file.fileName),
    writtenFiles,
    diagnostics: ts.getPreEmitDiagnostics(program),
  };
}

function emittableSources(program: ReturnType<TsApi['createProgram']>): TsSourceFile[] {
  return program
    .getSourceFiles()
    .filter(
      (file) =>
        !file.isDeclarationFile &&
        !program.isSourceFileFromExternalLibrary(file) &&
        !program.isSourceFileDefaultLibrary(file),
    );
}
