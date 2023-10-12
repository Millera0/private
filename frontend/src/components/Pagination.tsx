import {
  Flex,
  IconButton,
  Icon,
  IconButtonProps,
  Text,
  FlexProps,
  NumberInput,
  NumberInputField
} from '@chakra-ui/react';
import { useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { TbChevronsLeft, TbChevronLeft, TbChevronRight, TbChevronsRight } from 'react-icons/tb';

export interface PaginationProps extends FlexProps {
  pageCount: number;

  pageIndex: number;
  setPageIndex: (pageIndex: number) => void;

  canPreviousPage: boolean;
  previousPage: () => void;

  canNextPage: boolean;
  nextPage: () => void;
}

export default function Pagination({
  pageCount,
  pageIndex,
  setPageIndex,
  canPreviousPage,
  previousPage,
  canNextPage,
  nextPage,
  ...props
}: PaginationProps) {
  const { t } = useTranslation();
  const jumpToPageInput = useRef<HTMLInputElement>(null);

  if (pageCount === 0) return <Flex {...props} gap={1} justify="flex-end" wrap="wrap" />;

  return (
    <Flex {...props} gap={1} justify="flex-end" wrap="wrap">
      <IconButton
        aria-label={t('table.first_page')}
        title={t('table.first_page') ?? ''}
        icon={<Text>{1}</Text>}
        display={{ base: 'none', md: 'inline-flex' }}
        onClick={() => setPageIndex(0)}
        disabled={!canPreviousPage}
      />

      <IconButton
        aria-label={t('table.previous_page')}
        title={t('table.previous_page') ?? ''}
        icon={<Icon as={TbChevronLeft} />}
        onClick={() => previousPage()}
        disabled={!canPreviousPage}
      />
      {[pageIndex - 2, pageIndex - 1, pageIndex, pageIndex + 1, pageIndex + 2]
        .filter((i) => i >= 0 && i < pageCount)
        .map((idx) => {
          const title = t('table.page', { page: idx + 1 });
          const disabled = pageIndex === idx;
          const style: Partial<IconButtonProps> = disabled ? { colorScheme: 'blue' } : {};
          return (
            <IconButton
              aria-label={title}
              title={title}
              key={idx}
              icon={<Text>{idx + 1}</Text>}
              onClick={() => setPageIndex(idx)}
              disabled={disabled}
              {...style}
            />
          );
        })}
      <IconButton
        aria-label={t('table.next_page')}
        title={t('table.next_page') ?? ''}
        icon={<Icon as={TbChevronRight} />}
        onClick={() => nextPage()}
        disabled={!canNextPage}
      />
      <IconButton
        aria-label={t('table.last_page')}
        title={t('table.last_page') ?? ''}
        icon={<Text>{pageCount}</Text>}
        display={{ base: 'none', md: 'inline-flex' }}
        onClick={() => setPageIndex(pageCount - 1)}
        disabled={!canNextPage}
      />
      <NumberInput
        defaultValue={pageIndex + 1}
        key={`${pageIndex + 1}`}
        inputMode="text"
        max={pageCount}
        min={1}
        maxW={16}
        allowMouseWheel
        onKeyUp={(e) => {
          if (e.key !== 'Enter') return;
          if (jumpToPageInput.current?.value == null) return;
          setPageIndex(+jumpToPageInput.current?.value - 1);
        }}
      >
        <NumberInputField ref={jumpToPageInput} px={2} textAlign="center" />
      </NumberInput>
    </Flex>
  );
}
