import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { mount } from '@vue/test-utils';
import GitAnalyzer from '../src/components/GitAnalyzer.vue';

describe('GitAnalyzer Component', () => {
  let wrapper: any;

  beforeEach(() => {
    (global as any).window.__TAURI__ = {
      invoke: vi.fn(),
      dialog: { save: vi.fn() }
    };
  });

  afterEach(() => {
    if (wrapper) wrapper.unmount();
  });

  it('renders correctly', () => {
    wrapper = mount(GitAnalyzer);
    expect(wrapper.find('h1').text()).toBe('Git 导出器');
    expect(wrapper.find('#projectPath').exists()).toBe(true);
  });

  it('allows project path input', async () => {
    wrapper = mount(GitAnalyzer);
    const input = wrapper.find('#projectPath');
    await input.setValue('/test/path');
    expect((wrapper.vm as any).projectPath).toBe('/test/path');
  });

  it('loads authors when button is clicked', async () => {
    const mockAuthors = [
      { name: 'Test User', email: 'test@example.com', count: 5 }
    ];
    (global.window as any).__TAURI__.invoke = vi.fn().mockResolvedValue(mockAuthors);

    wrapper = mount(GitAnalyzer);
    await wrapper.setData({ projectPath: '/test/path' });
    const loadButton = wrapper.find('button').at(0);
    await loadButton.trigger('click');
    expect((global.window as any).__TAURI__.invoke).toHaveBeenCalledWith('get_git_authors', {
      projectPath: '/test/path'
    });
  });

  it('shows author dropdown and allows selection', async () => {
    wrapper = mount(GitAnalyzer);
    await wrapper.setData({
      authors: [
        { name: 'Test User', email: 'test@example.com', count: 5 }
      ]
    });

    const searchInput = wrapper.find('input[type="text"]');
    await searchInput.trigger('focus');
    expect((wrapper.vm as any).showDropdown).toBe(true);

    // Toggle author
    const checkbox = wrapper.find('input[type="checkbox"]');
    if (checkbox.exists()) {
      await checkbox.setChecked();
      expect((wrapper.vm as any).selectedAuthors).toHaveLength(1);
    }
  });

  it('filters authors by search', async () => {
    wrapper = mount(GitAnalyzer);
    await wrapper.setData({
      authors: [
        { name: 'Alice', email: 'alice@test.com', count: 5 },
        { name: 'Bob', email: 'bob@test.com', count: 3 }
      ]
    });

    const searchInput = wrapper.find('input[type="text"]');
    await searchInput.setValue('alice');
    const filtered = (wrapper.vm as any).filteredAuthors;
    expect(filtered).toHaveLength(1);
    expect(filtered[0].name).toBe('Alice');
  });

  it('has date mode tab', async () => {
    wrapper = mount(GitAnalyzer);
    expect((wrapper.vm as any).dateMode).toBe('single');

    const tabs = wrapper.findAll('button');
    const rangeTab = tabs.find(btn => btn.text() === '时间段');
    if (rangeTab) {
      await rangeTab.trigger('click');
      expect((wrapper.vm as any).dateMode).toBe('range');
    }
  });

  it('validates export button disabled state', () => {
    wrapper = mount(GitAnalyzer);
    const exportButton = wrapper.findAll('button').find(btn => btn.text() === '导出为 TXT');
    expect(exportButton?.attributes('disabled')).toBeDefined();
  });
});
