// UI strings. Add a language: copy a block, translate it, and list it in LANGS.
const LANGS = { es: 'Español', en: 'English', pt: 'Português', zh: '中文', ja: '日本語', ko: '한국어' };
const STR = {
  en: { on: 'All cores awake', off: 'Windows parking', hint_on: 'Click to restore defaults', hint_off: 'Click to unpark all cores',
    cores: 'Cores', parked: '{n} parked', guard: 'Guardian', guard_sub: '· re-applies if reverted', guard_tip: 'Re-applies the setting if Windows reverts it',
    auto: 'Start with Windows', auto_tip: 'Starts hidden in the tray', bg: 'Keep running in background', bg_sub: '· on close',
    bg_tip: 'If off, closing the window quits Coreflow and the guardian stops', procs: 'Unpark while these processes run', procs_ph: 'game.exe, render.exe',
    lang: 'Language', theme: 'Theme', t_auto: 'Auto', t_dark: 'Dark', t_light: 'Light', ac: 'Plugged in {v}', dc: 'Battery {v}',
    bk_yes: 'Backup saved', bk_no: 'No backup', core_p: 'Core {i} · parked', core_u: 'Core {i} · {u}% · {g} GHz',
    min: 'Minimize', close: 'Close', settings: 'Settings', sw: 'Toggle unpark', m_open: 'Open Coreflow', m_toggle: 'Unpark cores', m_quit: 'Quit', m_on: 'All awake', m_off: 'Parking on' },
  es: { on: 'Todo despierto', off: 'Parking de Windows', hint_on: 'Pulsa para restaurar los valores', hint_off: 'Pulsa para desaparcar todos',
    cores: 'Núcleos', parked: '{n} aparcados', guard: 'Guardián', guard_sub: '· reaplica si se revierte', guard_tip: 'Reaplica el ajuste si Windows lo revierte',
    auto: 'Iniciar con Windows', auto_tip: 'Arranca oculto en la bandeja', bg: 'Seguir en segundo plano', bg_sub: '· al cerrar',
    bg_tip: 'Si lo desactivas, cerrar la ventana cierra Coreflow y el guardián se detiene', procs: 'Desaparcar con estos procesos', procs_ph: 'juego.exe, render.exe',
    lang: 'Idioma', theme: 'Tema', t_auto: 'Auto', t_dark: 'Oscuro', t_light: 'Claro', ac: 'Enchufado {v}', dc: 'Batería {v}',
    bk_yes: 'Copia guardada', bk_no: 'Sin copia', core_p: 'Núcleo {i} · aparcado', core_u: 'Núcleo {i} · {u}% · {g} GHz',
    min: 'Minimizar', close: 'Cerrar', settings: 'Ajustes', sw: 'Activar o desactivar unpark', m_open: 'Abrir Coreflow', m_toggle: 'Desaparcar núcleos', m_quit: 'Salir', m_on: 'Todo despierto', m_off: 'Parking activo' },
  pt: { on: 'Todos os núcleos ativos', off: 'Parking do Windows', hint_on: 'Clique para restaurar os valores', hint_off: 'Clique para desestacionar todos',
    cores: 'Núcleos', parked: '{n} estacionados', guard: 'Guardião', guard_sub: '· reaplica se for revertido', guard_tip: 'Reaplica o ajuste se o Windows o reverter',
    auto: 'Iniciar com o Windows', auto_tip: 'Inicia oculto na bandeja', bg: 'Manter em segundo plano', bg_sub: '· ao fechar',
    bg_tip: 'Se desativado, fechar a janela encerra o Coreflow e o guardião para', procs: 'Desestacionar enquanto estes processos rodarem', procs_ph: 'jogo.exe, render.exe',
    lang: 'Idioma', theme: 'Tema', t_auto: 'Auto', t_dark: 'Escuro', t_light: 'Claro', ac: 'Na tomada {v}', dc: 'Bateria {v}',
    bk_yes: 'Backup salvo', bk_no: 'Sem backup', core_p: 'Núcleo {i} · estacionado', core_u: 'Núcleo {i} · {u}% · {g} GHz',
    min: 'Minimizar', close: 'Fechar', settings: 'Configurações', sw: 'Alternar unpark', m_open: 'Abrir Coreflow', m_toggle: 'Desestacionar núcleos', m_quit: 'Sair', m_on: 'Todos ativos', m_off: 'Parking ativo' },
  zh: { on: '所有核心已唤醒', off: 'Windows 核心停放', hint_on: '点击恢复默认值', hint_off: '点击取消所有核心停放',
    cores: '核心', parked: '{n} 个已停放', guard: '守护', guard_sub: '· 被还原时重新应用', guard_tip: '当 Windows 还原设置时重新应用',
    auto: '开机自启', auto_tip: '启动后隐藏在托盘', bg: '后台继续运行', bg_sub: '· 关闭窗口时',
    bg_tip: '关闭后，关闭窗口将退出 Coreflow，守护也会停止', procs: '以下进程运行时取消停放', procs_ph: 'game.exe, render.exe',
    lang: '语言', theme: '主题', t_auto: '自动', t_dark: '深色', t_light: '浅色', ac: '接通电源 {v}', dc: '电池 {v}',
    bk_yes: '已备份', bk_no: '无备份', core_p: '核心 {i} · 已停放', core_u: '核心 {i} · {u}% · {g} GHz',
    min: '最小化', close: '关闭', settings: '设置', sw: '切换取消停放', m_open: '打开 Coreflow', m_toggle: '取消核心停放', m_quit: '退出', m_on: '全部唤醒', m_off: '停放中' },
  ja: { on: '全コアが稼働中', off: 'Windows のパーキング', hint_on: 'クリックで元の値に戻す', hint_off: 'クリックですべてのコアのパークを解除',
    cores: 'コア', parked: '{n} 個パーク中', guard: 'ガーディアン', guard_sub: '· 元に戻されたら再適用', guard_tip: 'Windows が設定を元に戻したら再適用します',
    auto: 'Windows と同時に起動', auto_tip: 'トレイに隠れた状態で起動します', bg: 'バックグラウンドで実行', bg_sub: '· 閉じたとき',
    bg_tip: 'オフにすると、ウィンドウを閉じた時に Coreflow が終了しガーディアンも停止します', procs: 'これらのプロセスの実行中はパーク解除', procs_ph: 'game.exe, render.exe',
    lang: '言語', theme: 'テーマ', t_auto: '自動', t_dark: 'ダーク', t_light: 'ライト', ac: '電源接続 {v}', dc: 'バッテリー {v}',
    bk_yes: 'バックアップ済み', bk_no: 'バックアップなし', core_p: 'コア {i} · パーク中', core_u: 'コア {i} · {u}% · {g} GHz',
    min: '最小化', close: '閉じる', settings: '設定', sw: 'パーク解除の切り替え', m_open: 'Coreflow を開く', m_toggle: 'コアのパーク解除', m_quit: '終了', m_on: '全コア稼働', m_off: 'パーキング中' },
  ko: { on: '모든 코어 활성', off: 'Windows 파킹', hint_on: '클릭하여 기본값 복원', hint_off: '클릭하여 모든 코어 파킹 해제',
    cores: '코어', parked: '{n}개 파킹됨', guard: '가디언', guard_sub: '· 되돌려지면 다시 적용', guard_tip: 'Windows가 설정을 되돌리면 다시 적용합니다',
    auto: 'Windows와 함께 시작', auto_tip: '트레이에 숨겨진 상태로 시작합니다', bg: '백그라운드에서 계속 실행', bg_sub: '· 창을 닫을 때',
    bg_tip: '끄면 창을 닫을 때 Coreflow가 종료되고 가디언도 멈춥니다', procs: '다음 프로세스 실행 중 파킹 해제', procs_ph: 'game.exe, render.exe',
    lang: '언어', theme: '테마', t_auto: '자동', t_dark: '어두움', t_light: '밝음', ac: '전원 연결 {v}', dc: '배터리 {v}',
    bk_yes: '백업됨', bk_no: '백업 없음', core_p: '코어 {i} · 파킹됨', core_u: '코어 {i} · {u}% · {g} GHz',
    min: '최소화', close: '닫기', settings: '설정', sw: '파킹 해제 전환', m_open: 'Coreflow 열기', m_toggle: '코어 파킹 해제', m_quit: '종료', m_on: '모두 활성', m_off: '파킹 중' },
};

let lang = 'en';
const t = (k, v = {}) => (STR[lang][k] ?? STR.en[k] ?? k).replace(/\{(\w+)\}/g, (_, n) => v[n] ?? '');

// cfg.lang / cfg.theme: 'auto' follows the system
function applyPrefs(cfg) {
  const sys = (navigator.language || 'en').slice(0, 2).toLowerCase();
  lang = STR[cfg.lang] ? cfg.lang : STR[sys] ? sys : 'en';
  document.documentElement.lang = lang;
  if (cfg.theme === 'dark' || cfg.theme === 'light') document.documentElement.dataset.theme = cfg.theme;
  else delete document.documentElement.dataset.theme;
  document.querySelectorAll('[data-i]').forEach(e => { e.textContent = t(e.dataset.i); });
  document.querySelectorAll('[data-i-title]').forEach(e => { e.title = t(e.dataset.iTitle); });
  document.querySelectorAll('[data-i-ph]').forEach(e => { e.placeholder = t(e.dataset.iPh); });
  document.querySelectorAll('[data-i-aria]').forEach(e => { e.setAttribute('aria-label', t(e.dataset.iAria)); });
}
