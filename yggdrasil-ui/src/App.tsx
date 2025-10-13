import { FormEvent, useEffect, useMemo, useState } from 'react';
import init, { WasmEngine } from './wasmLoader';

type EvalWarning = { toggle_name: string; message: string };

type ToggleSummary = {
  name: string;
  project: string;
  enabled: boolean;
  feature_type?: string | null;
};

type VariantResult = {
  name: string;
  payload?: unknown;
  enabled: boolean;
  featureEnabled: boolean;
};

type EvaluationResult = {
  enabled: boolean;
  variant: VariantResult;
};

type GrammarMap = Record<string, string>;

const DEFAULT_CONTEXT = '{\n  "userId": ""\n}';

function parseWarnings(value: unknown): EvalWarning[] {
  if (!Array.isArray(value)) {
    return [];
  }

  return value.flatMap((entry) => {
    if (!entry || typeof entry !== 'object') {
      return [];
    }

    const raw = entry as Partial<EvalWarning>;
    if (typeof raw.toggle_name === 'string' && typeof raw.message === 'string') {
      return [{ toggle_name: raw.toggle_name, message: raw.message }];
    }

    return [];
  });
}

function parseToggles(value: unknown): ToggleSummary[] {
  if (!Array.isArray(value)) {
    return [];
  }

  return value.flatMap((entry) => {
    if (!entry || typeof entry !== 'object') {
      return [];
    }

    const raw = entry as Partial<ToggleSummary>;
    if (typeof raw.name !== 'string' || typeof raw.project !== 'string' || typeof raw.enabled !== 'boolean') {
      return [];
    }

    return [
      {
        name: raw.name,
        project: raw.project,
        enabled: raw.enabled,
        feature_type: raw.feature_type ?? null,
      },
    ];
  });
}

function parseEvaluation(value: unknown): EvaluationResult | null {
  if (!value || typeof value !== 'object') {
    return null;
  }

  const raw = value as Partial<EvaluationResult>;
  if (typeof raw.enabled !== 'boolean' || !raw.variant || typeof raw.variant !== 'object') {
    return null;
  }

  const variant = raw.variant as Partial<VariantResult> & { feature_enabled?: boolean };
  const featureEnabled =
    typeof variant.featureEnabled === 'boolean'
      ? variant.featureEnabled
      : typeof variant.feature_enabled === 'boolean'
      ? variant.feature_enabled
      : null;

  if (typeof variant.name !== 'string' || typeof variant.enabled !== 'boolean' || featureEnabled === null) {
    return null;
  }

  return {
    enabled: raw.enabled,
    variant: {
      name: variant.name,
      enabled: variant.enabled,
      featureEnabled,
      payload: variant.payload,
    },
  };
}

function ensureRecordOfStrings(value: unknown): value is GrammarMap {
  if (typeof value !== 'object' || value === null) {
    return false;
  }
  return Object.entries(value).every(([, v]) => typeof v === 'string');
}

const sectionStyle: React.CSSProperties = {
  background: '#ffffff',
  borderRadius: 12,
  padding: '1.5rem',
  boxShadow: '0 10px 30px rgba(15, 23, 42, 0.08)',
  border: '1px solid rgba(148, 163, 184, 0.2)',
};

const labelStyle: React.CSSProperties = {
  display: 'block',
  fontWeight: 600,
  marginBottom: 6,
  color: '#0f172a',
};

const inputStyle: React.CSSProperties = {
  width: '100%',
  padding: '0.55rem 0.65rem',
  borderRadius: 8,
  border: '1px solid rgba(148, 163, 184, 0.55)',
  fontSize: '0.95rem',
};

const textareaStyle: React.CSSProperties = {
  ...inputStyle,
  minHeight: '160px',
  fontFamily: 'ui-monospace, SFMono-Regular, SFMono, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace',
};

const buttonBase: React.CSSProperties = {
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  gap: '0.35rem',
  padding: '0.55rem 1rem',
  borderRadius: 8,
  border: 'none',
  fontSize: '0.95rem',
  fontWeight: 600,
  cursor: 'pointer',
};

const primaryButton: React.CSSProperties = {
  ...buttonBase,
  background: '#2563eb',
  color: '#ffffff',
};

const secondaryButton: React.CSSProperties = {
  ...buttonBase,
  background: '#e2e8f0',
  color: '#0f172a',
};

const pillStyle: React.CSSProperties = {
  padding: '0.2rem 0.5rem',
  borderRadius: 999,
  fontSize: '0.75rem',
  background: '#e0f2fe',
  color: '#0369a1',
  fontWeight: 600,
  display: 'inline-flex',
  alignItems: 'center',
  gap: '0.2rem',
};

function App() {
  const [engine, setEngine] = useState<WasmEngine | null>(null);
  const [initError, setInitError] = useState<string | null>(null);
  const [initializing, setInitializing] = useState(true);

  const [unleashUrl, setUnleashUrl] = useState('');
  const [apiToken, setApiToken] = useState('');
  const [fetching, setFetching] = useState(false);
  const [fetchError, setFetchError] = useState<string | null>(null);
  const [warnings, setWarnings] = useState<EvalWarning[]>([]);

  const [grammars, setGrammars] = useState<GrammarMap>({});
  const [grammarText, setGrammarText] = useState('');
  const [grammarError, setGrammarError] = useState<string | null>(null);

  const [toggles, setToggles] = useState<ToggleSummary[]>([]);
  const [selectedToggle, setSelectedToggle] = useState('');

  const [contextJson, setContextJson] = useState(DEFAULT_CONTEXT);
  const [evaluationResult, setEvaluationResult] = useState<EvaluationResult | null>(null);
  const [evaluationError, setEvaluationError] = useState<string | null>(null);

  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        await init();
        if (!cancelled) {
          setEngine(new WasmEngine());
        }
      } catch (err) {
        if (!cancelled) {
          setInitError(err instanceof Error ? err.message : String(err));
        }
      } finally {
        if (!cancelled) {
          setInitializing(false);
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, []);

  const sortedToggleNames = useMemo(() => toggles.map((toggle) => toggle.name).sort(), [toggles]);

  useEffect(() => {
    if (!selectedToggle && sortedToggleNames.length > 0) {
      setSelectedToggle(sortedToggleNames[0]);
    }
  }, [sortedToggleNames, selectedToggle]);

  const handleFetchFeatures = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!engine || !unleashUrl) {
      return;
    }
    setFetching(true);
    setFetchError(null);
    setGrammarError(null);
    setEvaluationError(null);
    setStatusMessage(null);

    try {
      const response = await fetch(unleashUrl, {
        headers: apiToken
          ? {
              Authorization: apiToken,
            }
          : undefined,
      });

      if (!response.ok) {
        throw new Error(`Failed to fetch features (${response.status} ${response.statusText})`);
      }

      const payload = await response.text();
      const warningList = parseWarnings(engine.loadClientFeatures(payload));
      const grammarValue = engine.getGrammars();
      const grammarMap = ensureRecordOfStrings(grammarValue) ? grammarValue : {};
      const toggleList = parseToggles(engine.listToggles());

      setWarnings(warningList);
      setGrammars(grammarMap);
      setGrammarText(JSON.stringify(grammarMap, null, 2));
      setToggles(toggleList);
      setSelectedToggle(toggleList[0]?.name ?? '');
      setStatusMessage(`Loaded ${toggleList.length} toggles`);
      setEvaluationResult(null);
    } catch (err) {
      setFetchError(err instanceof Error ? err.message : String(err));
    } finally {
      setFetching(false);
    }
  };

  const handleApplyGrammar = () => {
    if (!engine) {
      return;
    }
    setGrammarError(null);
    setStatusMessage(null);

    try {
      const parsed = JSON.parse(grammarText);
      if (!ensureRecordOfStrings(parsed)) {
        throw new Error('Grammar payload must be an object whose values are strings.');
      }
      engine.setGrammars(parsed);
      setGrammars(parsed);
      setStatusMessage('Updated grammar overrides.');
    } catch (err) {
      setGrammarError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleEvaluate = () => {
    if (!engine || !selectedToggle) {
      return;
    }
    setEvaluationError(null);
    setStatusMessage(null);

    try {
      const result = parseEvaluation(engine.evaluate(selectedToggle, contextJson));
      if (!result) {
        throw new Error('Engine returned an unexpected evaluation payload.');
      }
      setEvaluationResult(result);
    } catch (err) {
      setEvaluationError(err instanceof Error ? err.message : String(err));
      setEvaluationResult(null);
    }
  };

  if (initializing) {
    return (
      <div style={{ padding: '4rem', textAlign: 'center', color: '#475569' }}>
        <p>Loading Yggdrasil engine…</p>
      </div>
    );
  }

  if (initError || !engine) {
    return (
      <div style={{ padding: '4rem', maxWidth: 680, margin: '0 auto' }}>
        <h1 style={{ color: '#b91c1c' }}>Failed to initialise WebAssembly</h1>
        <p>{initError ?? 'Unknown initialisation error.'}</p>
        <p style={{ color: '#475569' }}>
          Ensure the wasm artifacts are built with <code>npm run wasm:dev</code> before starting the UI.
        </p>
      </div>
    );
  }

  return (
    <div style={{ maxWidth: 1080, margin: '0 auto', padding: '2.5rem 1.5rem 4rem', display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      <header>
        <h1 style={{ margin: 0, fontSize: '2rem', color: '#0f172a' }}>Yggdrasil Playground</h1>
        <p style={{ marginTop: '0.5rem', color: '#475569' }}>
          Fetch feature configuration from your Unleash instance, inspect the compiled grammar, tweak it, and evaluate feature toggles on demand.
        </p>
      </header>

      <section style={sectionStyle}>
        <form onSubmit={handleFetchFeatures} style={{ display: 'grid', gap: '1rem' }}>
          <div>
            <label style={labelStyle} htmlFor="unleash-url">Unleash API URL</label>
            <input
              id="unleash-url"
              style={inputStyle}
              type="url"
              required
              placeholder="https://unleash.example.com/api/client/features"
              value={unleashUrl}
              onChange={(event) => setUnleashUrl(event.target.value)}
            />
          </div>

          <div>
            <label style={labelStyle} htmlFor="api-token">API Token</label>
            <input
              id="api-token"
              style={inputStyle}
              type="text"
              placeholder="Optional client token"
              value={apiToken}
              onChange={(event) => setApiToken(event.target.value)}
            />
          </div>

          <div style={{ display: 'flex', gap: '0.75rem', alignItems: 'center' }}>
            <button style={primaryButton} type="submit" disabled={fetching}>
              {fetching ? 'Fetching…' : 'Fetch feature configuration'}
            </button>
            {statusMessage && <span style={pillStyle}>{statusMessage}</span>}
          </div>
        </form>

        {fetchError && (
          <p style={{ color: '#b91c1c', marginTop: '1rem' }}>{fetchError}</p>
        )}
      </section>

      {warnings.length > 0 && (
        <section style={sectionStyle}>
          <h2 style={{ marginTop: 0, color: '#b45309' }}>Compilation warnings</h2>
          <ul style={{ paddingLeft: '1.25rem', color: '#92400e' }}>
            {warnings.map((warning) => (
              <li key={`${warning.toggle_name}-${warning.message}`}>[{warning.toggle_name}] {warning.message}</li>
            ))}
          </ul>
        </section>
      )}

      {Object.keys(grammars).length > 0 && (
        <section style={sectionStyle}>
          <h2 style={{ marginTop: 0, color: '#0f172a' }}>Grammar Overrides</h2>
          <p style={{ color: '#475569' }}>
            Edit the grammar map (JSON) and apply to recompile strategies inside the WebAssembly engine.
          </p>
          <textarea
            style={{ ...textareaStyle, minHeight: '280px' }}
            value={grammarText}
            onChange={(event) => setGrammarText(event.target.value)}
          />
          <div style={{ display: 'flex', gap: '0.75rem', marginTop: '0.75rem' }}>
            <button style={secondaryButton} type="button" onClick={handleApplyGrammar}>
              Apply grammar overrides
            </button>
            <button
              style={secondaryButton}
              type="button"
              onClick={() => setGrammarText(JSON.stringify(grammars, null, 2))}
            >
              Reset editor
            </button>
          </div>
          {grammarError && <p style={{ color: '#b91c1c', marginTop: '0.75rem' }}>{grammarError}</p>}
        </section>
      )}

      {sortedToggleNames.length > 0 && (
        <section style={sectionStyle}>
          <h2 style={{ marginTop: 0, color: '#0f172a' }}>Evaluate Toggle</h2>
          <div style={{ display: 'grid', gap: '1rem', marginBottom: '1rem' }}>
            <div>
              <label style={labelStyle} htmlFor="toggle-select">Toggle</label>
              <select
                id="toggle-select"
                style={inputStyle}
                value={selectedToggle}
                onChange={(event) => setSelectedToggle(event.target.value)}
              >
                {sortedToggleNames.map((name) => (
                  <option key={name} value={name}>
                    {name}
                  </option>
                ))}
              </select>
            </div>

            <div>
              <label style={labelStyle} htmlFor="context-json">Context JSON</label>
              <textarea
                id="context-json"
                style={textareaStyle}
                value={contextJson}
                onChange={(event) => setContextJson(event.target.value)}
              />
            </div>
          </div>

          <button style={primaryButton} type="button" onClick={handleEvaluate}>
            Evaluate
          </button>

          {evaluationError && <p style={{ color: '#b91c1c', marginTop: '0.75rem' }}>{evaluationError}</p>}

          {evaluationResult && (
            <div style={{ marginTop: '1rem', background: '#f8fafc', borderRadius: 8, padding: '1rem' }}>
              <p style={{ margin: 0, fontWeight: 600, color: '#0f172a' }}>
                {selectedToggle}: {evaluationResult.enabled ? 'ENABLED' : 'DISABLED'}
              </p>
              <p style={{ margin: '0.25rem 0', color: '#475569' }}>
                Variant: <strong>{evaluationResult.variant.name}</strong>
              </p>
              {evaluationResult.variant.payload && (
                <pre
                  style={{
                    margin: 0,
                    marginTop: '0.5rem',
                    padding: '0.75rem',
                    background: '#e2e8f0',
                    borderRadius: 6,
                    fontSize: '0.85rem',
                    overflowX: 'auto',
                  }}
                >
                  {JSON.stringify(evaluationResult.variant.payload, null, 2)}
                </pre>
              )}
            </div>
          )}
        </section>
      )}
    </div>
  );
}

export default App;
